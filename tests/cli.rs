use std::str::FromStr;
use wazir_drop::CliCommand;

#[test]
fn test_cli_command_display_from_str() {
    let test_cases = [
        "Time 1000",
        "Opening WNAADADAFFAADDAA wnaadadaffaaddaa",
        "Start",
        "a1a2",
        "Quit",
    ];
    for case in test_cases {
        println!("case: {case}");
        let command = CliCommand::from_str(case).unwrap();
        assert_eq!(command.to_string(), case);
    }
}
