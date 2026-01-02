use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::time::Duration;
use url::Url;

pub const DNS_TIMEOUT: Duration = Duration::from_secs(5);
pub const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
pub const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024;

const BLOCKED_HOSTS: &[&str] = &[
    "localhost",
    "127.0.0.1",
    "::1",
    "[::1]",
    "0.0.0.0",
    "metadata.google.internal",
    "metadata.goog",
    "169.254.169.254",
];

const BLOCKED_HOST_SUFFIXES: &[&str] = &[
    ".localhost",
    ".metadata.google.internal",
    ".metadata.goog",
];

pub async fn validate_url_for_fetch(url_str: &str) -> Result<(Url, SocketAddr), String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

    let scheme = url.scheme().to_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(format!(
            "Unsupported scheme '{}': only http and https are allowed",
            scheme
        ));
    }

    let host = url
        .host_str()
        .ok_or_else(|| "URL must have a host".to_string())?;

    if let Some(ip) = normalize_ip_format(host) {
        if is_private_or_restricted_ip(ip) {
            return Err(format!(
                "Access to private/restricted IP {} is not allowed",
                ip
            ));
        }
    }

    let host_lower = host.to_lowercase();
    for blocked in BLOCKED_HOSTS {
        if host_lower == *blocked {
            return Err(format!("Access to '{}' is not allowed", host));
        }
    }

    for suffix in BLOCKED_HOST_SUFFIXES {
        if host_lower.ends_with(suffix) {
            return Err(format!(
                "Access to '{}' is not allowed (blocked suffix)",
                host
            ));
        }
    }

    let port = url.port_or_known_default().unwrap_or(match scheme.as_str() {
        "https" => 443,
        _ => 80,
    });

    let addr_str = format!("{}:{}", host, port);

    let resolved_addrs: Vec<SocketAddr> = tokio::time::timeout(DNS_TIMEOUT, async {
        tokio::task::spawn_blocking(move || addr_str.to_socket_addrs().map(|iter| iter.collect()))
            .await
            .map_err(|e| format!("DNS resolution task failed: {}", e))?
            .map_err(|e| format!("DNS resolution failed: {}", e))
    })
    .await
    .map_err(|_| "DNS resolution timed out".to_string())??;

    if resolved_addrs.is_empty() {
        return Err("DNS resolution returned no addresses".to_string());
    }

    for addr in &resolved_addrs {
        if is_private_or_restricted_ip(addr.ip()) {
            return Err(format!(
                "DNS resolved to private/restricted IP {} which is not allowed",
                addr.ip()
            ));
        }
    }

    let pinned_addr = resolved_addrs[0];

    Ok((url, pinned_addr))
}

pub fn is_private_or_restricted_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => is_restricted_ipv4(ipv4),
        IpAddr::V6(ipv6) => is_restricted_ipv6(ipv6),
    }
}

pub fn is_restricted_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();

    if ip.is_loopback() {
        return true;
    }

    if ip.is_private() {
        return true;
    }

    if ip.is_link_local() {
        return true;
    }

    if ip.is_broadcast() {
        return true;
    }

    if octets[0] == 0 {
        return true;
    }

    if octets[0] == 100 && (octets[1] >= 64 && octets[1] <= 127) {
        return true;
    }

    if octets[0] == 198 && (octets[1] == 18 || octets[1] == 19) {
        return true;
    }

    if octets[0] == 192 && octets[1] == 0 && octets[2] == 2 {
        return true;
    }
    if octets[0] == 198 && octets[1] == 51 && octets[2] == 100 {
        return true;
    }
    if octets[0] == 203 && octets[1] == 0 && octets[2] == 113 {
        return true;
    }

    if octets[0] == 169 && octets[1] == 254 {
        return true;
    }

    false
}

pub fn is_restricted_ipv6(ip: Ipv6Addr) -> bool {
    if ip.is_loopback() {
        return true;
    }

    if ip.is_unspecified() {
        return true;
    }

    let segments = ip.segments();

    if segments[0] == 0
        && segments[1] == 0
        && segments[2] == 0
        && segments[3] == 0
        && segments[4] == 0
        && segments[5] == 0xffff
    {
        let ipv4 = Ipv4Addr::new(
            (segments[6] >> 8) as u8,
            segments[6] as u8,
            (segments[7] >> 8) as u8,
            segments[7] as u8,
        );
        return is_restricted_ipv4(ipv4);
    }

    if segments[0] == 0
        && segments[1] == 0
        && segments[2] == 0
        && segments[3] == 0
        && segments[4] == 0
        && segments[5] == 0
    {
        let last_segment = segments[6] as u32 * 65536 + segments[7] as u32;
        if last_segment != 0 && last_segment != 1 {
            let ipv4 = Ipv4Addr::new(
                (segments[6] >> 8) as u8,
                segments[6] as u8,
                (segments[7] >> 8) as u8,
                segments[7] as u8,
            );
            if is_restricted_ipv4(ipv4) {
                return true;
            }
        }
    }

    if segments[0] == 0x64 && segments[1] == 0xff9b && segments[2] == 0 && segments[3] == 0 {
        let ipv4 = Ipv4Addr::new(
            (segments[6] >> 8) as u8,
            segments[6] as u8,
            (segments[7] >> 8) as u8,
            segments[7] as u8,
        );
        return is_restricted_ipv4(ipv4);
    }

    if segments[0] == 0x2002 {
        let ipv4 = Ipv4Addr::new(
            (segments[1] >> 8) as u8,
            segments[1] as u8,
            (segments[2] >> 8) as u8,
            segments[2] as u8,
        );
        return is_restricted_ipv4(ipv4);
    }

    if segments[0] == 0x2001 && segments[1] == 0 {
        return true;
    }

    if (segments[0] & 0xfe00) == 0xfc00 {
        return true;
    }

    if (segments[0] & 0xffc0) == 0xfe80 {
        return true;
    }

    false
}

pub fn normalize_ip_format(host: &str) -> Option<IpAddr> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Some(ip);
    }

    if let Some(ip) = parse_decimal_ip(host) {
        return Some(IpAddr::V4(ip));
    }

    if let Some(ip) = parse_dotted_mixed_format(host) {
        return Some(IpAddr::V4(ip));
    }

    None
}

fn parse_decimal_ip(host: &str) -> Option<Ipv4Addr> {
    let num: u32 = host.parse().ok()?;
    Some(Ipv4Addr::from(num))
}

fn parse_dotted_mixed_format(host: &str) -> Option<Ipv4Addr> {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() != 4 {
        return None;
    }

    let mut octets = [0u8; 4];
    for (i, part) in parts.iter().enumerate() {
        let value = parse_ip_octet(part)?;
        if value > 255 {
            return None;
        }
        octets[i] = value as u8;
    }

    Some(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]))
}

pub fn parse_ip_octet(s: &str) -> Option<u32> {
    if s.is_empty() {
        return None;
    }

    if s.starts_with("0x") || s.starts_with("0X") {
        return u32::from_str_radix(&s[2..], 16).ok();
    }

    if s.starts_with('0') && s.len() > 1 && s.chars().all(|c| c.is_ascii_digit()) {
        return u32::from_str_radix(s, 8).ok();
    }

    s.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restricted_ipv4_loopback() {
        assert!(is_restricted_ipv4(Ipv4Addr::new(127, 0, 0, 1)));
        assert!(is_restricted_ipv4(Ipv4Addr::new(127, 255, 255, 255)));
    }

    #[test]
    fn test_restricted_ipv4_private() {
        assert!(is_restricted_ipv4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(is_restricted_ipv4(Ipv4Addr::new(172, 16, 0, 1)));
        assert!(is_restricted_ipv4(Ipv4Addr::new(192, 168, 1, 1)));
    }

    #[test]
    fn test_restricted_ipv4_link_local() {
        assert!(is_restricted_ipv4(Ipv4Addr::new(169, 254, 169, 254)));
    }

    #[test]
    fn test_restricted_ipv4_cgnat() {
        assert!(is_restricted_ipv4(Ipv4Addr::new(100, 64, 0, 1)));
        assert!(is_restricted_ipv4(Ipv4Addr::new(100, 127, 255, 255)));
    }

    #[test]
    fn test_restricted_ipv4_benchmark() {
        assert!(is_restricted_ipv4(Ipv4Addr::new(198, 18, 0, 1)));
        assert!(is_restricted_ipv4(Ipv4Addr::new(198, 19, 255, 255)));
    }

    #[test]
    fn test_allowed_ipv4() {
        assert!(!is_restricted_ipv4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(!is_restricted_ipv4(Ipv4Addr::new(1, 1, 1, 1)));
    }

    #[test]
    fn test_restricted_ipv6_loopback() {
        assert!(is_restricted_ipv6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)));
    }

    #[test]
    fn test_restricted_ipv6_mapped_ipv4() {
        assert!(is_restricted_ipv6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0xffff, 0x7f00, 0x0001
        )));
    }

    #[test]
    fn test_restricted_ipv6_ula() {
        assert!(is_restricted_ipv6(Ipv6Addr::new(
            0xfc00, 0, 0, 0, 0, 0, 0, 1
        )));
        assert!(is_restricted_ipv6(Ipv6Addr::new(
            0xfd00, 0, 0, 0, 0, 0, 0, 1
        )));
    }

    #[test]
    fn test_restricted_ipv6_link_local() {
        assert!(is_restricted_ipv6(Ipv6Addr::new(
            0xfe80, 0, 0, 0, 0, 0, 0, 1
        )));
    }

    #[test]
    fn test_normalize_decimal_ip() {
        let ip = normalize_ip_format("2130706433");
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
    }

    #[test]
    fn test_normalize_octal_ip() {
        let ip = normalize_ip_format("0177.0.0.01");
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
    }

    #[test]
    fn test_normalize_hex_ip() {
        let ip = normalize_ip_format("0x7f.0x0.0x0.0x1");
        assert_eq!(ip, Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
    }

    #[test]
    fn test_parse_ip_octet_decimal() {
        assert_eq!(parse_ip_octet("127"), Some(127));
    }

    #[test]
    fn test_parse_ip_octet_octal() {
        assert_eq!(parse_ip_octet("0177"), Some(127));
    }

    #[test]
    fn test_parse_ip_octet_hex() {
        assert_eq!(parse_ip_octet("0x7f"), Some(127));
    }
}
