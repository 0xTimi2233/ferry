//! Ferry 服务端入口程序

use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_execute_main_successfully() {
        let code = main();
        assert_eq!(code, ExitCode::SUCCESS);
    }
}
