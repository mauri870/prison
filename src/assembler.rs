const RULESET: &str = include_str!("../isa.asm");

pub fn assemble(src: &str) -> Result<Vec<u8>, String> {
    let combined = format!("{}\n{}", RULESET, src);
    let filename = "input.asm";

    let mut report = customasm::diagn::Report::new();
    let mut fileserver = customasm::util::FileServerMock::new();
    fileserver.add(filename, combined.as_str());

    let opts = customasm::asm::AssemblyOptions::new();
    let assembly = customasm::asm::assemble(
        &mut report,
        &opts,
        &mut fileserver,
        &[filename],
    );

    if report.has_errors() {
        let mut buf = Vec::new();
        report.print_all(&mut buf, &fileserver, false);
        return Err(String::from_utf8_lossy(&buf).into_owned());
    }

    Ok(assembly.output
        .map(|o| o.format_binary(&mut report))
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assemble_play_cooperate() {
        let bytes = assemble("PLAY COOPERATE").unwrap();
        assert_eq!(bytes, &[0x70]);
    }

    #[test]
    fn assemble_play_defect() {
        let bytes = assemble("PLAY DEFECT").unwrap();
        assert_eq!(bytes, &[0x71]);
    }

    #[test]
    fn assemble_error_returns_err() {
        assert!(assemble("NOT_AN_INSTRUCTION").is_err());
    }
}
