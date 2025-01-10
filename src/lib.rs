#[macro_use]
mod macros; // this should be previous than the mod phy; fuck rust

pub mod phy;
pub mod time;
pub mod wire;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
