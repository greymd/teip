mod cmdtest {
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
        cmd.args(&["-l", "2,4-5", "sed", "s/./@/"])
            .write_stdin("111\n222\n333\n444\n555\n666\n")
            .assert()
            .stdout("111\n@22\n333\n@44\n@55\n666\n");
    }

    #[test]
    fn test_regex_line() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-g", "[AB]", "sed", "s/./@/"])
            .write_stdin("ABC\nDFE\nBCC\nCCA\n")
            .assert()
            .stdout("@BC\nDFE\n@CC\n@CA\n");
    }

    #[test]
    fn test_regex_only() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-og", "2", "sed", "s/./A/"])
            .write_stdin("118\n119\n120\n121\n")
            .assert()
            .stdout("118\n119\n1A0\n1A1\n");
    }

    #[test]
    fn test_regex_only_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-og", "\\d+", "-v", "tr", "[:upper:]", "[:lower:]"])
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
            "perl",
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
        cmd.args(&["-zv","-og", "^...", "tr", "[:alnum:]", "@"])
            .write_stdin("ABC123EFG\0HIJKLM456")
            .assert()
            .stdout("ABC@@@@@@\0HIJ@@@@@@");
    }

    #[test]
    fn test_regex_only_multiple() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-og", "\\d", "sed", "s/./AA/g"])
            .write_stdin("120\n121\n")
            .assert()
            .stdout("AAAAAA\nAAAAAA\n");
    }

    #[test]
    fn test_solid_line() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-l", "2,4-5", "sed", "s/./@/"])
            .write_stdin("111\n222\n333\n444\n555\n666\n")
            .assert()
            .stdout("111\n@22\n333\n@44\n@55\n666\n");
    }

    #[test]
    fn test_solid_regex_line() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-g", "[AB]", "sed", "s/./@/"])
            .write_stdin("ABC\nDFE\nBCC\nCCA\n")
            .assert()
            .stdout("@BC\nDFE\n@CC\n@CA\n");
    }

    #[test]
    fn test_solid_regex_only() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-og", "2", "sed", "s/./A/"])
            .write_stdin("118\n119\n120\n121\n")
            .assert()
            .stdout("118\n119\n1A0\n1A1\n");
    }

    #[test]
    fn test_solid_regex_only_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-og", "\\d+", "-v", "tr", "[:upper:]", "[:lower:]"])
            .write_stdin("ABC123EFG\nHIJKLM456")
            .assert()
            .stdout("abc123efg\nhijklm456");
    }

    #[test]
    fn test_solid_regex_only_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sv","-og", "\\d+", "tr", "[:upper:]", "[:lower:]"])
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
            "perl",
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
        cmd.args(&["-sz", "-og", "(..\\n..|F.G)", "--", "tr", "-dc", "."])
            .write_stdin("ABC\nDEF\0GHI\nJKL")
            .assert()
            .stdout("AF\0GL");
    }

    #[test]
    fn test_character_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-zvc", "1", "--", "tr", "[:alnum:]", "@"])
            .write_stdin("ABC\nDEF\n\0GHI\nJKL")
            .assert()
            .stdout("A@@\n@@@\n\0G@@\n@@@");
    }

    #[test]
    fn test_solid_regex_only_multiple() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-og", "\\d", "sed", "s/./AA/g"])
            .write_stdin("120\n121\n")
            .assert()
            .stdout("AAAAAA\nAAAAAA\n");
    }

    #[test]
    fn test_onig() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-Gog", "\\d+(?=D)", "sed", "s/./@/g"])
            .write_stdin("ABC123DEF456\n")
            .assert()
            .stdout("ABC@@@DEF456\n");
    }

    #[test]
    fn test_onig_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-v","-Gog","\\d+(?=D)", "sed", "s/./@/g"])
            .write_stdin("ABC123DEF456\n")
            .assert()
            .stdout("@@@123@@@@@@\n");
    }

    #[test]
    fn test_onig_null() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        // Use perl -0 instead of sed -z because BSD does not support it.
        cmd.args(&[
            "-z",
            "-Gog",
            ".\\n.",
            "--",
            "perl",
            "-0",
            "-pnle",
            "s/^./@/;s/.$/%/;",
        ])
        .write_stdin("ABC\nDEF\nGHI\nJKL\n")
        .assert()
        .stdout("AB@\n%E@\n%H@\n%KL\n");
    }

    #[test]
    fn test_onig_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        // Use perl -0 instead of sed -z because BSD does not support it.
        cmd.args(&["-zv","-Gog", "^...", "tr", "[:alnum:]", "@"])
            .write_stdin("ABC123EFG\0HIJKLM456")
            .assert()
            .stdout("ABC@@@@@@\0HIJ@@@@@@");
    }

    #[test]
    fn test_onig_multiple() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-Gog", "C\\K\\d+(?=D)", "sed", "s/./@/g"])
            .write_stdin("ABC123DEF456\nEFG123ABC456DEF\n")
            .assert()
            .stdout("ABC@@@DEF456\nEFG123ABC@@@DEF\n");
    }

    #[test]
    fn test_solid_onig() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-Gog", "2", "sed", "s/./A/"])
            .write_stdin("118\n119\n120\n121\n")
            .assert()
            .stdout("118\n119\n1A0\n1A1\n");
    }

    #[test]
    fn test_solid_onig_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-Gog", "\\d+", "-v", "tr", "[:upper:]", "[:lower:]"])
            .write_stdin("ABC123EFG\nHIJKLM456")
            .assert()
            .stdout("abc123efg\nhijklm456");
    }

    #[test]
    fn test_solid_onig_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sv","-Gog", "\\d+", "tr", "[:upper:]", "[:lower:]"])
            .write_stdin("ABC123EFG\0\nHIJKLM456")
            .assert()
            .stdout("abc123efg\0\nhijklm456");
    }

    #[test]
    fn test_solid_onig_null() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&[
            "-sz",
            "-Gog",
            ".\\n.",
            "--",
            "perl",
            "-pne",
            "$. == 2 and printf \"_\"",
        ])
        .write_stdin("ABC\nDEF\nGHI\nJKL\n")
        .assert()
        .stdout("ABC\n_DEF\n_GHI\n_JKL\n");
    }

    #[test]
    fn test_solid_onig_null2() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sz", "-Gog", "(..\\n..|F.G)", "--", "tr", "-dc", "."])
            .write_stdin("ABC\nDEF\0GHI\nJKL")
            .assert()
            .stdout("AF\0GL");
    }

    #[test]
    fn test_character_range() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-c", "1-3,6-8", "sed", "s/./A/"])
            .write_stdin("111111111\n222222222\n")
            .assert()
            .stdout("A1111A111\nA2222A222\n");
    }

    #[test]
    fn test_character_range_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-c", "1,4-6", "-v", "tr", "[:upper:]", "[:lower:]"])
            .write_stdin("ABCEFG\nHIJKLM")
            .assert()
            .stdout("AbcEFG\nHijKLM");
    }

    #[test]
    fn test_character_separate() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-c", "1,2,4", "sed", "s/./A/"])
            .write_stdin("1234\n")
            .assert()
            .stdout("A23A\n"); // Not "AA3A"
    }

    #[test]
    fn test_character_be_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-c", "1,2,4", "grep", "2"])
            .write_stdin("1234\n")
            .assert()
            .stdout("123")
            .code(1);
    }

    #[test]
    fn test_solid_character_range() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-c", "1-3,6-8", "sed", "s/./A/"])
            .write_stdin("111111111\n222222222\n")
            .assert()
            .stdout("A1111A111\nA2222A222\n");
    }

    #[test]
    fn test_solid_character_range_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-c", "1,4-6", "-v", "tr", "[:upper:]", "[:lower:]"])
            .write_stdin("ABCEFG\nHIJKLM")
            .assert()
            .stdout("AbcEFG\nHijKLM");
    }

    #[test]
    fn test_solid_character_null() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sz", "-c", "1", "--", "tr", "[:alnum:]", "@"])
            .write_stdin("ABC\nDEF\n\0GHI\nJKL")
            .assert()
            .stdout("@BC\nDEF\n\0@HI\nJKL");
    }

    #[test]
    fn test_solid_character_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sz", "-v", "-c", "1", "--", "tr", "[:alnum:]", "@"])
            .write_stdin("ABC\nDEF\n\0GHI\nJKL")
            .assert()
            .stdout("A@@\n@@@\n\0G@@\n@@@");
    }

    #[test]
    fn test_solid_character_separate() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-c", "1,2,4", "sed", "s/./A/"])
            .write_stdin("1234\n")
            .assert()
            .stdout("A23A\n"); // Not "AA3A"
    }

    #[test]
    fn test_solid_character_be_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-c", "1,2,4", "grep", "2"])
            .write_stdin("1234\n")
            .assert()
            .stdout("123\n");
    }

    #[test]
    fn test_field() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-d", ",", "-f", "1,2,4", "sed", "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("_AA,_BB,CCC,_DD\n_EE,_FF,GGG,_HH\n");
    }

    #[test]
    fn test_field_range() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-d", ",", "-f", "2-4", "sed", "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,_BB,_CC,_DD\nEEE,_FF,_GG,_HH\n");
    }

    #[test]
    fn test_field_range_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-v", "-d", ",", "-f", "2-4", "sed", "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("_AA,BBB,CCC,DDD\n_EE,FFF,GGG,HHH\n");
    }

    #[test]
    fn test_field_range_to_last() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-d", ",", "-f", "3-", "sed", "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,BBB,_CC,_DD\nEEE,FFF,_GG,_HH\n");
    }

    #[test]
    fn test_field_be_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-d", ",", "-f", "3-", "grep", "G"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,BBB,GGG,")
            .code(1);
    }

    // This case may be failed in case of debug version somehow. Try release version.
    #[test]
    fn test_field_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-d", ",", "-f", "3-", "seq", "5"])
            .write_stdin("AAA,BBB,CCC,,\nEEE,,GGG,\n")
            .assert()
            .stdout("AAA,BBB,1,2,3\nEEE,,4,5\n");
    }

    #[test]
    fn test_field_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-vz", "-f", "2", "-d", ",", "tr", "#", "@"])
            .write_stdin("1#,2#\n,3#,\0 4#,5#,#6")
            .assert()
            .stdout("1@,2#\n,3@,\0 4@,5#,@6");
    }

    #[test]
    fn test_field_ws() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-f", "1-2,4,5", "--", "awk", "{s+=$0; print s}"])
            .write_stdin("1 2 3 4 5\n")
            .assert()
            .stdout("1 3 3 7 12\n");
    }

    #[test]
    fn test_field_ws_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-f", "3-5", "--", "sed", "s/.*/@@@/g"])
            .write_stdin("  2\t 3 4 \t  \n")
            .assert()
            .stdout("  2\t @@@ @@@ \t  @@@\n");
    }

    #[test]
    fn test_field_regex_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-f", "3-5", "-D", "[ \\t]+", "--", "awk", "{print \"@@@\"}"])
            .write_stdin("  2\t 3 4 \t  \n")
            .assert()
            .stdout("  2\t @@@ @@@ \t  @@@\n");
    }

    #[test]
    fn test_field_ws_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-f", "1-3,6", "--", "awk", "{print $0*2}"])
            .write_stdin("   1  \t 2 \t\t\t3   4\t5\n")
            .assert()
            .stdout("0   2  \t 4 \t\t\t3   4\t10\n");
    }

    #[test]
    fn test_solid_field() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-d", ",", "-f", "1,2,4", "sed", "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("_AA,_BB,CCC,_DD\n_EE,_FF,GGG,_HH\n");
    }

    #[test]
    fn test_solid_field_range() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-d", ",", "-f", "2-4", "sed", "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,_BB,_CC,_DD\nEEE,_FF,_GG,_HH\n");
    }

    #[test]
    fn test_solid_field_range_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-v", "-d", ",", "-f", "2-4", "sed", "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("_AA,BBB,CCC,DDD\n_EE,FFF,GGG,HHH\n");
    }

    #[test]
    fn test_solid_field_null() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-sz", "-f", "2", "-d", ",", "tr", "#", "@"])
            .write_stdin("1#,2#\n,3#,\0 4#,5#,#6")
            .assert()
            .stdout("1#,2@\n,3#,\0 4#,5@,#6");
    }

    #[test]
    fn test_solid_field_null_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-vsz", "-f", "2", "-d", ",", "tr", "#", "@"])
            .write_stdin("1#,2#\n,3#,\0 4#,5#,#6")
            .assert()
            .stdout("1@,2#\n,3@,\0 4@,5#,@6");
    }

    #[test]
    fn test_solid_field_range_to_last() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-d", ",", "-f", "3-", "sed", "s/./_/"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,BBB,_CC,_DD\nEEE,FFF,_GG,_HH\n");
    }

    #[test]
    fn test_solid_field_be_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-d", ",", "-f", "3-", "grep", "G"])
            .write_stdin("AAA,BBB,CCC,DDD\nEEE,FFF,GGG,HHH\n")
            .assert()
            .stdout("AAA,BBB,,\nEEE,FFF,GGG,\n");
    }

    #[test]
    fn test_solid_field_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-d", ",", "-f", "3-", "grep", "."])
            .write_stdin("AAA,BBB,CCC,,\nEEE,,GGG,\n")
            .assert()
            .stdout("AAA,BBB,CCC,,\nEEE,,GGG,\n");
    }

    #[test]
    fn test_solid_field_ws() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-f", "1-2,4,5", "--", "awk", "{s+=$0; print s}"])
            .write_stdin("1 2 3 4 5\n")
            .assert()
            .stdout("1 2 3 4 5\n");
    }

    #[test]
    fn test_solid_field_ws_invert() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-f", "1-3,6", "--", "awk", "{print $0*2}"])
            .write_stdin("   1  \t 2 \t\t\t3   4\t5\n")
            .assert()
            .stdout("0   2  \t 4 \t\t\t3   4\t10\n");
    }

    #[test]
    fn test_solid_field_ws_empty() {
        let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(&["-s", "-f", "3-5", "--", "awk", "{print \"@@@\"}"])
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
            "awk",
            "{print \"@@@\"}",
        ])
        .write_stdin("  2\t 3 4 \t  \n")
        .assert()
        .stdout("  2\t @@@ @@@ \t  @@@\n");
    }
}
