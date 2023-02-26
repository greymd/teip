mod cmdtest {
    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            static SED_CMD: &str = "C:\\Program Files\\Git\\usr\\bin\\sed.exe";
            static TR_CMD: &str = "C:\\Program Files\\Git\\usr\\bin\\tr.exe";
            static AWK_CMD: &str = "C:\\Program Files\\Git\\usr\\bin\\awk.exe";
            static PERL_CMD: &str = "C:\\Program Files\\Git\\usr\\bin\\perl.exe";
            static SEQ_CMD: &str = "C:\\Program Files\\Git\\usr\\bin\\seq.exe";
            static GREP_CMD: &str = "C:\\Program Files\\Git\\usr\\bin\\grep.exe";
            static _NL_CMD: &str = "C:\\Program Files\\Git\\usr\\bin\\nl.exe";
            static _ECHO_CMD: &str = "C:\\Program Files\\Git\\usr\\bin\\echo.exe";
            static ESCAPE_ECHO_CMD: &str = "C:\\\"Program Files\"\\Git\\usr\\bin\\echo.exe";
            static ESCAPE_NL_CMD: &str = "C:\\\"Program Files\"\\Git\\usr\\bin\\nl.exe";
            static ESCAPE_GREP_CMD: &str = "C:\\\"Program Files\"\\Git\\usr\\bin\\grep.exe";
        } else {
            static SED_CMD: &str = "sed";
            static TR_CMD: &str = "tr";
            static AWK_CMD: &str = "awk";
            static PERL_CMD: &str = "perl";
            static SEQ_CMD: &str = "seq";
            static GREP_CMD: &str = "grep";
            static _NL_CMD: &str = "nl";
            static _ECHO_CMD: &str = "echo";
            static ESCAPE_ECHO_CMD: &str = "echo";
            static ESCAPE_NL_CMD: &str = "nl";
            static ESCAPE_GREP_CMD: &str = "grep";
        }
    }

    #[test]
    fn test_no_argument() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.write_stdin("test\n")
            .assert()
            .code(1);
    }

    #[test]
    fn test_exoffload_grep() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let mcmd = format!("{} -n A", ESCAPE_GREP_CMD);
        cmd.args(&["-e", &mcmd, SED_CMD, "s/./@/"])
            .write_stdin("ABC\nDFE\nBCC\nCCA\n")
            .assert()
            .stdout("@BC\nDFE\nBCC\n@CA\n");
    }

    #[test]
    fn test_exoffload_grep_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let mcmd = format!("{} -n A", ESCAPE_GREP_CMD);
        cmd.args(&["-e", &mcmd, "-v", SED_CMD, "s/./@/"])
            .write_stdin("ABC\nDFE\nBCC\nCCA\n")
            .assert()
            .stdout("ABC\n@FE\n@CC\nCCA\n");
    }

    #[test]
    fn test_exoffload_nl() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let mcmd = format!("{}", ESCAPE_NL_CMD);
        cmd.args(&["-e", &mcmd, SED_CMD, "s/./@/g"])
            .write_stdin("ABC\nDFE\nBCC\nCCA\n")
            .assert()
            .stdout("@@@\n@@@\n@@@\n@@@\n");
    }

    #[test]
    fn test_exoffload_nl_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let mcmd = format!("{}", ESCAPE_NL_CMD);
        cmd.args(&["-v", "-e", &mcmd, SED_CMD, "s/./@/g"])
            .write_stdin("ABC\nDFE\nBCC\nCCA\n")
            .assert()
            .stdout("ABC\nDFE\nBCC\nCCA\n");
    }

    #[test]
    fn test_exoffload_nl_solid() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let mcmd = format!("{}", ESCAPE_NL_CMD);
        cmd.args(&["-e", &mcmd, "-s", SED_CMD, "s/./@/g"])
            .write_stdin("ABC\nDFE\nBCC\nCCA\n")
            .assert()
            .stdout("@@@\n@@@\n@@@\n@@@\n");
    }

    #[test]
    fn test_exoffload_echo() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let mcmd = format!("{} 3", ESCAPE_ECHO_CMD);
        cmd.args(&["-e", &mcmd, SED_CMD, "s/./@/g"])
            .write_stdin("ABC\nDFE\nBCC\nCCA\n")
            .assert()
            .stdout("ABC\nDFE\n@@@\nCCA\n");
    }

    #[test]
    fn test_exoffload_echo_solid() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let mcmd = format!("{} 3", ESCAPE_ECHO_CMD);
        cmd.args(&["-s", "-e", &mcmd, SED_CMD, "s/.//g"])
            .write_stdin("ABC\nDFE\nBCC\nCCA\n")
            .assert()
            .stdout("ABC\nDFE\n\nCCA\n");
    }

    #[test]
    fn test_character_range_error_c() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-c", "2-1"])
            .write_stdin("test\n")
            .assert()
            .code(1);
    }

    #[test]
    fn test_line() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-l", "2,4-5", SED_CMD, "s/./@/"])
            .write_stdin("111\n222\n333\n444\n555\n666\n")
            .assert()
            .stdout("111\n@22\n333\n@44\n@55\n666\n");
    }

    #[test]
    fn test_regex_line() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-g", "[AB]", SED_CMD, "s/./@/"])
            .write_stdin("ABC\nDFE\nBCC\nCCA\n")
            .assert()
            .stdout("@BC\nDFE\n@CC\n@CA\n");
    }

    #[test]
    fn test_regex_only() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-og", "2", SED_CMD, "s/./A/"])
            .write_stdin("118\n119\n120\n121\n")
            .assert()
            .stdout("118\n119\n1A0\n1A1\n");
    }

    #[test]
    fn test_regex_only_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-og", "\\d+", "-v", TR_CMD, "[:upper:]", "[:lower:]"])
            .write_stdin("ABC123EFG\nHIJKLM456")
            .assert()
            .stdout("abc123efg\nhijklm456");
    }

    #[test]
    fn test_regex_only_null() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        // Use perl -0 instead of sed -z because BSD does not support it.
        cmd.args(&[
            "-z",
            "-og",
            ".\\n.",
            "--",
            PERL_CMD,
            "-0",
            "-pnle",
            "s/^./@/;s/.$/%/;",
        ])
        .write_stdin("ABC\nDEF\nGHI\nJKL\n")
        .assert()
        .stdout("AB@\n%E@\n%H@\n%KL\n");
    }

    #[test]
    fn test_regex_only_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        // Use perl -0 instead of sed -z because BSD does not support it.
        cmd.args(&["-zv", "-og", "^...", TR_CMD, "[:alnum:]", "@"])
            .write_stdin("ABC123EFG\0HIJKLM456")
            .assert()
            .stdout("ABC@@@@@@\0HIJ@@@@@@");
    }

    #[test]
    fn test_regex_only_multiple() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-og", "\\d", SED_CMD, "s/./AA/g"])
            .write_stdin("120\n121\n")
            .assert()
            .stdout("AAAAAA\nAAAAAA\n");
    }

    #[test]
    fn test_solid_line() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-l", "2,4-5", SED_CMD, "s/./@/"])
            .write_stdin("111\n222\n333\n444\n555\n666\n")
            .assert()
            .stdout("111\n@22\n333\n@44\n@55\n666\n");
    }

    #[test]
    fn test_solid_regex_line() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-g", "[AB]", SED_CMD, "s/./@/"])
            .write_stdin("ABC\nDFE\nBCC\nCCA\n")
            .assert()
            .stdout("@BC\nDFE\n@CC\n@CA\n");
    }

    #[test]
    fn test_solid_regex_only() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-og", "2", SED_CMD, "s/./A/"])
            .write_stdin("118\n119\n120\n121\n")
            .assert()
            .stdout("118\n119\n1A0\n1A1\n");
    }

    #[test]
    fn test_solid_regex_only_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-og", "\\d+", "-v", TR_CMD, "[:upper:]", "[:lower:]"])
            .write_stdin("ABC123EFG\nHIJKLM456")
            .assert()
            .stdout("abc123efg\nhijklm456");
    }

    #[test]
    fn test_solid_regex_only_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sv", "-og", "\\d+", TR_CMD, "[:upper:]", "[:lower:]"])
            .write_stdin("ABC123EFG\0\nHIJKLM456")
            .assert()
            .stdout("abc123efg\0\nhijklm456");
    }

    #[test]
    fn test_solid_regex_only_null() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&[
            "-sz",
            "-og",
            ".\\n.",
            "--",
            PERL_CMD,
            "-pne",
            "$. == 2 and printf \"_\"",
        ])
        .write_stdin("ABC\nDEF\nGHI\nJKL\n")
        .assert()
        .stdout("ABC\n_DEF\n_GHI\n_JKL\n");
    }

    #[test]
    fn test_solid_regex_only_null2() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sz", "-og", "(..\\n..|F.G)", "--", TR_CMD, "-dc", "."])
            .write_stdin("ABC\nDEF\0GHI\nJKL")
            .assert()
            .stdout("AF\0GL");
    }

    #[test]
    fn test_character_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-zvc", "1", "--", TR_CMD, "[:alnum:]", "@"])
            .write_stdin("ABC\nDEF\n\0GHI\nJKL")
            .assert()
            .stdout("A@@\n@@@\n\0G@@\n@@@");
    }

    #[test]
    fn test_solid_regex_only_multiple() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-og", "\\d", SED_CMD, "s/./AA/g"])
            .write_stdin("120\n121\n")
            .assert()
            .stdout("AAAAAA\nAAAAAA\n");
    }

    #[test]
    #[cfg(feature = "oniguruma")]
    fn test_onig() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-Gog", "\\d+(?=D)", SED_CMD, "s/./@/g"])
            .write_stdin("ABC123DEF456\n")
            .assert()
            .stdout("ABC@@@DEF456\n");
    }

    #[test]
    #[cfg(feature = "oniguruma")]
    fn test_onig_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-v", "-Gog", "\\d+(?=D)", SED_CMD, "s/./@/g"])
            .write_stdin("ABC123DEF456\n")
            .assert()
            .stdout("@@@123@@@@@@\n");
    }

    #[test]
    #[cfg(feature = "oniguruma")]
    fn test_onig_null() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        // Use perl -0 instead of sed -z because BSD does not support it.
        cmd.args(&[
            "-z",
            "-Gog",
            ".\\n.",
            "--",
            PERL_CMD,
            "-0",
            "-pnle",
            "s/^./@/;s/.$/%/;",
        ])
        .write_stdin("ABC\nDEF\nGHI\nJKL\n")
        .assert()
        .stdout("AB@\n%E@\n%H@\n%KL\n");
    }

    #[test]
    #[cfg(feature = "oniguruma")]
    fn test_onig_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        // Use perl -0 instead of sed -z because BSD does not support it.
        cmd.args(&["-zv", "-Gog", "^...", TR_CMD, "[:alnum:]", "@"])
            .write_stdin("ABC123EFG\0HIJKLM456")
            .assert()
            .stdout("ABC@@@@@@\0HIJ@@@@@@");
    }

    #[test]
    #[cfg(feature = "oniguruma")]
    fn test_onig_multiple() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-Gog", "C\\K\\d+(?=D)", SED_CMD, "s/./@/g"])
            .write_stdin("ABC123DEF456\nEFG123ABC456DEF\n")
            .assert()
            .stdout("ABC@@@DEF456\nEFG123ABC@@@DEF\n");
    }

    #[test]
    #[cfg(feature = "oniguruma")]
    fn test_solid_onig() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-Gog", "2", SED_CMD, "s/./A/"])
            .write_stdin("118\n119\n120\n121\n")
            .assert()
            .stdout("118\n119\n1A0\n1A1\n");
    }

    #[test]
    #[cfg(feature = "oniguruma")]
    fn test_solid_onig_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-Gog", "\\d+", "-v", TR_CMD, "[:upper:]", "[:lower:]"])
            .write_stdin("ABC123EFG\nHIJKLM456")
            .assert()
            .stdout("abc123efg\nhijklm456");
    }

    #[test]
    #[cfg(feature = "oniguruma")]
    fn test_solid_onig_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sv", "-Gog", "\\d+", TR_CMD, "[:upper:]", "[:lower:]"])
            .write_stdin("ABC123EFG\0\nHIJKLM456")
            .assert()
            .stdout("abc123efg\0\nhijklm456");
    }

    #[test]
    #[cfg(feature = "oniguruma")]
    fn test_solid_onig_null() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&[
            "-sz",
            "-Gog",
            ".\\n.",
            "--",
            PERL_CMD,
            "-pne",
            "$. == 2 and printf \"_\"",
        ])
        .write_stdin("ABC\nDEF\nGHI\nJKL\n")
        .assert()
        .stdout("ABC\n_DEF\n_GHI\n_JKL\n");
    }

    #[test]
    #[cfg(feature = "oniguruma")]
    fn test_solid_onig_null2() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sz", "-Gog", "(..\\n..|F.G)", "--", TR_CMD, "-dc", "."])
            .write_stdin("ABC\nDEF\0GHI\nJKL")
            .assert()
            .stdout("AF\0GL");
    }

    #[test]
    fn test_character_range() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-c", "1-3,6-8", SED_CMD, "s/./A/"])
            .write_stdin("111111111\n222222222\n")
            .assert()
            .stdout("A1111A111\nA2222A222\n");
    }

    #[test]
    fn test_character_range_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-c", "1,4-6", "-v", TR_CMD, "[:upper:]", "[:lower:]"])
            .write_stdin("ABCEFG\nHIJKLM")
            .assert()
            .stdout("AbcEFG\nHijKLM");
    }

    #[test]
    fn test_character_separate() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-c", "1,2,4", SED_CMD, "s/./A/"])
            .write_stdin("1234\n")
            .assert()
            .stdout("A23A\n"); // Not "AA3A"
    }

    #[test]
    fn test_character_be_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-c", "1,2,4", GREP_CMD, "2"])
            .write_stdin("1234\n")
            .assert()
            .stdout("123")
            .code(1);
    }

    #[test]
    fn test_solid_character_range() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-c", "1-3,6-8", SED_CMD, "s/./A/"])
            .write_stdin("111111111\n222222222\n")
            .assert()
            .stdout("A1111A111\nA2222A222\n");
    }

    #[test]
    fn test_solid_character_range_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-c", "1,4-6", "-v", TR_CMD, "[:upper:]", "[:lower:]"])
            .write_stdin("ABCEFG\nHIJKLM")
            .assert()
            .stdout("AbcEFG\nHijKLM");
    }

    #[test]
    fn test_solid_character_null() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sz", "-c", "1", "--", TR_CMD, "[:alnum:]", "@"])
            .write_stdin("ABC\nDEF\n\0GHI\nJKL")
            .assert()
            .stdout("@BC\nDEF\n\0@HI\nJKL");
    }

    #[test]
    fn test_solid_character_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sz", "-v", "-c", "1", "--", TR_CMD, "[:alnum:]", "@"])
            .write_stdin("ABC\nDEF\n\0GHI\nJKL")
            .assert()
            .stdout("A@@\n@@@\n\0G@@\n@@@");
    }

    #[test]
    fn test_solid_character_separate() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-c", "1,2,4", SED_CMD, "s/./A/"])
            .write_stdin("1234\n")
            .assert()
            .stdout("A23A\n"); // Not "AA3A"
    }

    #[test]
    fn test_solid_character_be_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-c", "1,2,4", GREP_CMD, "2"])
            .write_stdin("1234\n")
            .assert()
            .stdout("123\n");
    }

    #[test]
    fn test_field() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-d", ",", "-f", "1,2,4", SED_CMD, "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("_AA,_BB,CCC,_DD\n_EE,_FF,GGG,_HH\n");
    }

    #[test]
    fn test_field_range() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-d", ",", "-f", "2-4", SED_CMD, "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,_BB,_CC,_DD\nEEE,_FF,_GG,_HH\n");
    }

    #[test]
    fn test_field_range_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-v", "-d", ",", "-f", "2-4", SED_CMD, "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("_AA,BBB,CCC,DDD\n_EE,FFF,GGG,HHH\n");
    }

    #[test]
    fn test_field_range_to_last() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-d", ",", "-f", "3-", SED_CMD, "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,BBB,_CC,_DD\nEEE,FFF,_GG,_HH\n");
    }

    #[test]
    fn test_field_be_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-d", ",", "-f", "3-", GREP_CMD, "G"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,BBB,GGG,")
            .code(1);
    }

    // This case may be failed in case of debug version somehow. Try release version.
    #[test]
    fn test_field_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-d", ",", "-f", "3-", SEQ_CMD, "5"])
            .write_stdin("AAA,BBB,CCC,,\nEEE,,GGG,\n")
            .assert()
            .stdout("AAA,BBB,1,2,3\nEEE,,4,5\n");
    }

    #[test]
    fn test_field_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-vz", "-f", "2", "-d", ",", TR_CMD, "#", "@"])
            .write_stdin("1#,2#\n,3#,\0 4#,5#,#6")
            .assert()
            .stdout("1@,2#\n,3@,\0 4@,5#,@6");
    }

    #[test]
    fn test_field_ws() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-f", "1-2,4,5", "--", AWK_CMD, "{s+=$0; print s}"])
            .write_stdin("1 2 3 4 5\n")
            .assert()
            .stdout("1 3 3 7 12\n");
    }

    #[test]
    fn test_field_ws_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-f", "3-5", "--", SED_CMD, "s/.*/@@@/g"])
            .write_stdin("  2\t 3 4 \t  \n")
            .assert()
            .stdout("  2\t @@@ @@@ \t  @@@\n");
    }

    #[test]
    fn test_field_regex_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-f", "3-5", "-D", "[ \\t]+", "--", AWK_CMD, "{print \"@@@\"}"])
            .write_stdin("  2\t 3 4 \t  \n")
            .assert()
            .stdout("  2\t @@@ @@@ \t  @@@\n");
    }

    #[test]
    fn test_field_ws_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-f", "1-3,6", "--", AWK_CMD, "{print $0*2}"])
            .write_stdin("   1  \t 2 \t\t\t3   4\t5\n")
            .assert()
            .stdout("0   2  \t 4 \t\t\t3   4\t10\n");
    }

    #[test]
    fn test_csv() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        // test for UTF-8 as well to test the CSV parser
        // Do not use "sed" here because it does not support UTF-8 on Windows environment on GitHub.
        cmd.args(&["--csv", "-f", "2", "--", PERL_CMD, "-Mutf8", "-C", "-pe", "s/./@/g"])
            .write_stdin("名前,作者,ノート\n１レコード目,\"あいう\nえお\",かきく\n２レコード目,\"さしす\nせそ\",\"たちつ\nてと\"\n")
            .assert()
            .stdout("名前,@@,ノート\n１レコード目,@@@@\n@@@,かきく\n２レコード目,@@@@\n@@@,\"たちつ\nてと\"\n");
    }

    #[test]
    fn test_csv_end_nolf() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-f", "3", "--", PERL_CMD, "-Mutf8", "-C", "-pe", "s/./@/g"])
            .write_stdin("名前,作者,ノート\n１レコード目,\"あいう\nえお\",かきく\n２レコード目,\"さしす\nせそ\",\"たちつ\nてと\"")
            .assert()
            .stdout("名前,作者,@@@\n１レコード目,\"あいう\nえお\",@@@\n２レコード目,\"さしす\nせそ\",@@@@\n@@@");
    }

    #[test]
    fn test_csv_non_delim_comma() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-f", "1", "--", PERL_CMD, "-Mutf8", "-C", "-pe", "s/./@/g"])
            .write_stdin("\"名,前\",作者,ノート\n１レコード目,\"あいう\nえお\",かきく\n\"２レコード,目\",\"さしす\nせそ\",\"たちつ\nてと\"")
            .assert()
            .stdout("@@@@@,作者,ノート\n@@@@@@,\"あいう\nえお\",かきく\n@@@@@@@@@,\"さしす\nせそ\",\"たちつ\nてと\"");
    }

    #[test]
    fn test_csv_double_quotation() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-f", "2", "--", PERL_CMD, "-Mutf8", "-C", "-pe", "s/./@/g"])
            .write_stdin("名前,\"作\"\"者\",ノート\n１レコード目,\"あ\"\"いう\"\"\nえお\",かきく\n２レコード目,\"\"\"さしす\n\"\"せそ\",\"たちつ\nてと\"\n")
            .assert()
            .stdout("名前,@@@@@@,ノート\n１レコード目,@@@@@@@@\n@@@,かきく\n２レコード目,@@@@@@\n@@@@@,\"たちつ\nてと\"\n");
    }

    #[test]
    fn test_csv_crlf() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-f", "3", "--", TR_CMD, "\\r", "@"])
            .write_stdin("AAA,BBB,CCC\r\n1AAA,\"1BBB\r\nBB\",1CCC\r\n2AAA,2BBB,\"2CCC\r\nCC\"\r\n")
            .assert()
                 .stdout("AAA,BBB,CCC\r\n1AAA,\"1BBB\r\nBB\",1CCC\r\n2AAA,2BBB,\"2CCC@\nCC\"\r\n");
    }

    #[test]
    fn test_solid_field() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-d", ",", "-f", "1,2,4", SED_CMD, "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("_AA,_BB,CCC,_DD\n_EE,_FF,GGG,_HH\n");
    }

    #[test]
    fn test_solid_field_range() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-d", ",", "-f", "2-4", SED_CMD, "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,_BB,_CC,_DD\nEEE,_FF,_GG,_HH\n");
    }

    #[test]
    fn test_solid_field_range_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-v", "-d", ",", "-f", "2-4", SED_CMD, "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("_AA,BBB,CCC,DDD\n_EE,FFF,GGG,HHH\n");
    }

    #[test]
    fn test_solid_field_null() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sz", "-f", "2", "-d", ",", TR_CMD, "#", "@"])
            .write_stdin("1#,2#\n,3#,\0 4#,5#,#6")
            .assert()
            .stdout("1#,2@\n,3#,\0 4#,5@,#6");
    }

    #[test]
    fn test_solid_field_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-vsz", "-f", "2", "-d", ",", TR_CMD, "#", "@"])
            .write_stdin("1#,2#\n,3#,\0 4#,5#,#6")
            .assert()
            .stdout("1@,2#\n,3@,\0 4@,5#,@6");
    }

    #[test]
    fn test_solid_field_range_to_last() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-d", ",", "-f", "3-", SED_CMD, "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,BBB,_CC,_DD\nEEE,FFF,_GG,_HH\n");
    }

    #[test]
    fn test_solid_field_be_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-d", ",", "-f", "3-", GREP_CMD, "G"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,BBB,,\nEEE,FFF,GGG,\n");
    }

    #[test]
    fn test_solid_field_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-d", ",", "-f", "3-", GREP_CMD, "."])
            .write_stdin("AAA,BBB,CCC,,\nEEE,,GGG,\n")
            .assert()
            .stdout("AAA,BBB,CCC,,\nEEE,,GGG,\n");
    }

    #[test]
    fn test_solid_field_ws() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-f", "1-2,4,5", "--", AWK_CMD, "{s+=$0; print s}"])
            .write_stdin("1 2 3 4 5\n")
            .assert()
            .stdout("1 2 3 4 5\n");
    }

    #[test]
    fn test_solid_field_ws_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-f", "1-3,6", "--", AWK_CMD, "{print $0*2}"])
            .write_stdin("   1  \t 2 \t\t\t3   4\t5\n")
            .assert()
            .stdout("0   2  \t 4 \t\t\t3   4\t10\n");
    }

    #[test]
    fn test_solid_field_ws_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-f", "3-5", "--", AWK_CMD, "{print \"@@@\"}"])
            .write_stdin("  2\t 3 4 \t  \n")
            .assert()
            .stdout("  2\t @@@ @@@ \t  @@@\n");
    }

    #[test]
    fn test_solid_field_regex_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&[
            "-s",
            "-f",
            "3-5",
            "-D",
            "[ \\t]+",
            "--",
            AWK_CMD,
            "{print \"@@@\"}",
        ])
        .write_stdin("  2\t 3 4 \t  \n")
        .assert()
        .stdout("  2\t @@@ @@@ \t  @@@\n");
    }

    #[test]
    fn test_solid_csv() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-s", "-f", "2", "--", SED_CMD, "s/./@/g"])
            .write_stdin("名前,作者,ノート\n１レコード目,\"あいう\nえお\",かきく\n２レコード目,\"さしす\nせそ\",\"たちつ\nてと\"\n")
            .assert()
            .stdout("名前,@@,ノート\n１レコード目,@@@@\n@@@,かきく\n２レコード目,@@@@\n@@@,\"たちつ\nてと\"\n");
    }

    #[test]
    fn test_solid_csv_end_nolf() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-s", "-f", "3", "--", PERL_CMD, "-Mutf8", "-C", "-pe", "s/./@/g"])
            .write_stdin("名前,作者,ノート\n１レコード目,\"あいう\nえお\",かきく\n２レコード目,\"さしす\nせそ\",\"たちつ\nてと\"")
            .assert()
            .stdout("名前,作者,@@@\n１レコード目,\"あいう\nえお\",@@@\n２レコード目,\"さしす\nせそ\",@@@@\n@@@");
    }

    #[test]
    fn test_solid_csv_non_delim_comma() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-s", "-f", "1", "--", PERL_CMD, "-Mutf8", "-C", "-pe", "s/./@/g"])
            .write_stdin("\"名,前\",作者,ノート\n１レコード目,\"あいう\nえお\",かきく\n\"２レコード,目\",\"さしす\nせそ\",\"たちつ\nてと\"")
            .assert()
            .stdout("@@@@@,作者,ノート\n@@@@@@,\"あいう\nえお\",かきく\n@@@@@@@@@,\"さしす\nせそ\",\"たちつ\nてと\"");
    }

    #[test]
    fn test_solid_csv_double_quotation() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-s", "-f", "2", "--", PERL_CMD, "-Mutf8", "-C", "-pe", "s/./@/g"])
            .write_stdin("名前,\"作\"\"者\",ノート\n１レコード目,\"あ\"\"いう\"\"\nえお\",かきく\n２レコード目,\"\"\"さしす\n\"\"せそ\",\"たちつ\nてと\"\n")
            .assert()
            .stdout("名前,@@@@@@,ノート\n１レコード目,@@@@@@@@\n@@@,かきく\n２レコード目,@@@@@@\n@@@@@,\"たちつ\nてと\"\n");
    }

    #[test]
    fn test_solid_csv_crlf() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-s", "-f", "3", "--", TR_CMD, "\\r", "@"])
            .write_stdin("AAA,BBB,CCC\r\n1AAA,\"1BBB\r\nBB\",1CCC\r\n2AAA,2BBB,\"2CCC\r\nCC\"\r\n")
            .assert()
                 .stdout("AAA,BBB,CCC\r\n1AAA,\"1BBB\r\nBB\",1CCC\r\n2AAA,2BBB,\"2CCC@\nCC\"\r\n");
    }

    #[test]
    fn test_solid_csv_remove_newlines() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-s", "-f", "2", "--", TR_CMD, "-d", "\\n"])
            .write_stdin("名前,作者,ノート\n１レコード目,\"あい\nうえお\",かきく\n２レコード目,\"\"\"さしす\nせそ\",\"たちつ\nてと\"\n")
            .assert()
            .stdout("名前,作者,ノート\n１レコード目,\"あいうえお\",かきく\n２レコード目,\"\"\"さしすせそ\",\"たちつ\nてと\"\n");
    }

    #[test]
    fn test_solid_csv_chomp() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-s", "--chomp", "-f", "2,3", "--", TR_CMD, "\\n", "@"])
            .write_stdin("名前,作者,ノート\n１レコード目,\"あいう\nえお\",かきく\n２レコード目,\"さしす\nせそ\",\"たちつ\nてと,\n")
            .assert()
            .stdout("名前,作者,ノート\n１レコード目,\"あいう@えお\",かきく\n２レコード目,\"さしす@せそ\",\"たちつ@てと,");
    }

    #[test]
    fn test_solid_csv_nochomp() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["--csv", "-s", "-f", "2,3", "--", TR_CMD, "\\n", "@"])
            .write_stdin("名前,作者,ノート\n１レコード目,\"あいう\nえお\",かきく\n２レコード目,\"さしす\nせそ\",\"たちつ\nてと,\n")
            .assert()
            .stdout("名前,作者@,ノート@\n１レコード目,\"あいう@えお\"@,かきく@\n２レコード目,\"さしす@せそ\"@,\"たちつ@てと,@@");
    }

    #[test]
    fn test_solid_nochomp() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-o", "-g", "\\d+", "--", TR_CMD, "\\n", "@"])
            .write_stdin("AAA111BBB\nCCC222DDD\nEEE333FFF")
            .assert()
            .stdout("AAA111@BBB\nCCC222@DDD\nEEE333@FFF");
    }

    #[test]
    fn test_solid_chomp() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "--chomp", "-o", "-g", "\\d+", "--", TR_CMD, "\\n", "@"])
            .write_stdin("AAA111BBB\nCCC222DDD\nEEE333FFF")
            .assert()
            .stdout("AAA111BBB\nCCC222DDD\nEEE333FFF");
    }
}
