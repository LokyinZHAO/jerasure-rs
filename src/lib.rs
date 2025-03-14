pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use jerasure_sys;

    #[test]
    fn it_works() {
        unsafe {
            jerasure_sys::jerasure::galois_init_default_field(8);
        }
    }
}
