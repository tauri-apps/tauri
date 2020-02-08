pub fn pid() -> u32 {
    5
}

// Specifically dumb function, just to trigger clippy
pub fn pid_is_valid(pid: u32) -> bool {
    if pid < 5 {
        return false;
    } else {
        return true;
    }
}

#[cfg(test)]
mod tests {
    use std::process;

    use super::pid;

    #[test]
    fn test_pid_is_working() {
        assert_eq!(pid(), 5);
    }
}