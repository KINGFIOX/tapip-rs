use core::result;
use core::str::FromStr;

use crate::wire::EthernetAddress;
use crate::wire::{IpAddress, IpCidr, IpEndpoint};
use crate::wire::{Ipv4Address, Ipv4AddressExt, Ipv4Cidr};

type Result<T> = result::Result<T, ()>;

struct Parser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(data: &'a str) -> Parser<'a> {
        Parser {
            data: data.as_bytes(),
            pos: 0,
        }
    }

    #[allow(unused)]
    fn lookahead_char(&self, ch: u8) -> bool {
        if self.pos < self.data.len() {
            self.data[self.pos] == ch
        } else {
            false
        }
    }

    fn advance(&mut self) -> Result<u8> {
        match self.data.get(self.pos) {
            Some(&chr) => {
                self.pos += 1;
                Ok(chr)
            }
            None => Err(()),
        }
    }

    fn try_do<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Parser<'a>) -> Result<T>,
    {
        let pos = self.pos;
        match f(self) {
            Ok(res) => Some(res),
            Err(()) => {
                self.pos = pos;
                None
            }
        }
    }

    fn accept_eof(&mut self) -> Result<()> {
        if self.data.len() == self.pos {
            Ok(())
        } else {
            Err(())
        }
    }

    fn until_eof<F, T>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Parser<'a>) -> Result<T>,
    {
        let res = f(self)?;
        self.accept_eof()?;
        Ok(res)
    }

    fn accept_char(&mut self, chr: u8) -> Result<()> {
        if self.advance()? == chr {
            Ok(())
        } else {
            Err(())
        }
    }

    #[allow(unused)]
    fn accept_str(&mut self, string: &[u8]) -> Result<()> {
        for byte in string.iter() {
            self.accept_char(*byte)?;
        }
        Ok(())
    }

    fn accept_digit(&mut self, hex: bool) -> Result<u8> {
        let digit = self.advance()?;
        if digit.is_ascii_digit() {
            Ok(digit - b'0')
        } else if hex && (b'a'..=b'f').contains(&digit) {
            Ok(digit - b'a' + 10)
        } else if hex && (b'A'..=b'F').contains(&digit) {
            Ok(digit - b'A' + 10)
        } else {
            Err(())
        }
    }

    fn accept_number(&mut self, max_digits: usize, max_value: u32, hex: bool) -> Result<u32> {
        let mut value = self.accept_digit(hex)? as u32;
        for _ in 1..max_digits {
            match self.try_do(|p| p.accept_digit(hex)) {
                Some(digit) => {
                    value *= if hex { 16 } else { 10 };
                    value += digit as u32;
                }
                None => break,
            }
        }
        if value < max_value {
            Ok(value)
        } else {
            Err(())
        }
    }

    fn accept_mac_joined_with(&mut self, separator: u8) -> Result<EthernetAddress> {
        let mut octets = [0u8; 6];
        for (n, octet) in octets.iter_mut().enumerate() {
            *octet = self.accept_number(2, 0x100, true)? as u8;
            if n != 5 {
                self.accept_char(separator)?;
            }
        }
        Ok(EthernetAddress(octets))
    }

    fn accept_mac(&mut self) -> Result<EthernetAddress> {
        if let Some(mac) = self.try_do(|p| p.accept_mac_joined_with(b'-')) {
            return Ok(mac);
        }
        if let Some(mac) = self.try_do(|p| p.accept_mac_joined_with(b':')) {
            return Ok(mac);
        }
        Err(())
    }

    fn accept_ipv4_octets(&mut self) -> Result<[u8; 4]> {
        let mut octets = [0u8; 4];
        for (n, octet) in octets.iter_mut().enumerate() {
            *octet = self.accept_number(3, 0x100, false)? as u8;
            if n != 3 {
                self.accept_char(b'.')?;
            }
        }
        Ok(octets)
    }

    fn accept_ipv4(&mut self) -> Result<Ipv4Address> {
        let octets = self.accept_ipv4_octets()?;
        Ok(Ipv4Address::from_bytes(&octets))
    }

    fn accept_ip(&mut self) -> Result<IpAddress> {
        #[allow(clippy::single_match)]
        match self.try_do(|p| p.accept_ipv4()) {
            Some(ipv4) => return Ok(IpAddress::Ipv4(ipv4)),
            None => (),
        }

        Err(())
    }

    fn accept_ipv4_endpoint(&mut self) -> Result<IpEndpoint> {
        let ip = self.accept_ipv4()?;

        let port = if self.accept_eof().is_ok() {
            0
        } else {
            self.accept_char(b':')?;
            self.accept_number(5, 65535, false)?
        };

        Ok(IpEndpoint {
            addr: IpAddress::Ipv4(ip),
            port: port as u16,
        })
    }

    fn accept_ip_endpoint(&mut self) -> Result<IpEndpoint> {
        #[allow(clippy::single_match)]
        match self.try_do(|p| p.accept_ipv4_endpoint()) {
            Some(ipv4) => return Ok(ipv4),
            None => (),
        }

        Err(())
    }
}

impl FromStr for EthernetAddress {
    type Err = ();

    /// Parse a string representation of an Ethernet address.
    fn from_str(s: &str) -> Result<EthernetAddress> {
        Parser::new(s).until_eof(|p| p.accept_mac())
    }
}

impl FromStr for IpAddress {
    type Err = ();

    /// Parse a string representation of an IP address.
    fn from_str(s: &str) -> Result<IpAddress> {
        Parser::new(s).until_eof(|p| p.accept_ip())
    }
}

impl FromStr for Ipv4Cidr {
    type Err = ();

    /// Parse a string representation of an IPv4 CIDR.
    fn from_str(s: &str) -> Result<Ipv4Cidr> {
        Parser::new(s).until_eof(|p| {
            let ip = p.accept_ipv4()?;
            p.accept_char(b'/')?;
            let prefix_len = p.accept_number(2, 33, false)? as u8;
            Ok(Ipv4Cidr::new(ip, prefix_len))
        })
    }
}

impl FromStr for IpCidr {
    type Err = ();

    /// Parse a string representation of an IP CIDR.
    fn from_str(s: &str) -> Result<IpCidr> {
        #[allow(clippy::single_match)]
        match Ipv4Cidr::from_str(s) {
            Ok(cidr) => return Ok(IpCidr::Ipv4(cidr)),
            Err(_) => (),
        }

        Err(())
    }
}

impl FromStr for IpEndpoint {
    type Err = ();

    fn from_str(s: &str) -> Result<IpEndpoint> {
        Parser::new(s).until_eof(|p| p.accept_ip_endpoint())
    }
}
