use std::io::Result;

pub fn parse_line_sep(line: &str, separator: char) -> Result<Vec<String>> {
    let mut rv: Vec<String> = vec![];
    let mut in_quote = false;
    let mut buff: Vec<char> = vec![];
    let mut col_completed = false;
    let len = line.len();
    // parse column in csv separated comma line
    for (i, c) in line.chars().enumerate() {
        if col_completed {
            // wait until get comma
            if c == separator {
                col_completed = false;
            }
            continue;
        }
        if c == '"' {
            if in_quote {
                rv.push(buff.iter().collect());
                buff.clear();
                col_completed = true;
            }
            in_quote = !in_quote;
            continue;
        }
        if c == separator && !in_quote {
            rv.push(buff.iter().collect());
            buff.clear();
            if i == len - 1 {
                rv.push("".to_string());
            }
            continue;
        }
        buff.push(c);
        if i == len - 1 {
            rv.push(buff.iter().collect());
        }
    }
    Ok(rv.into_iter().map(|val| val.trim().to_string()).collect())
}

pub fn parse_line(line: &str) -> Result<Vec<String>> {
    parse_line_sep(line, ',')
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! gen_test {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let result = parse_line($input).unwrap();
                // assert_eq!(result.1.len(), $expected.len());
                assert_eq!(result, $expected);
            }
        };
    }

    gen_test!(test_empty1, "a,,c", vec!["a", "", "c"]);
    gen_test!(test_empty2, "a,,", vec!["a", "", ""]);
    gen_test!(test_empty3, ",,", vec!["", "", ""]);
    gen_test!(test_basic, "a,b,c", vec!["a", "b", "c"]);
    gen_test!(test_with_quote1, "a,\"b\",c", vec!["a", "b", "c"]);
    gen_test!(
        test_with_quote2,
        "a,\"b\",\"c\",d",
        vec!["a", "b", "c", "d"]
    );
    gen_test!(
        test_with_quote_first_col,
        "\"a\",b,c,d",
        vec!["a", "b", "c", "d"]
    );
    gen_test!(
        test_with_quote_ends_col,
        "a,b,c,\"d\"",
        vec!["a", "b", "c", "d"]
    );
    gen_test!(
        test_with_quote_contains_comma,
        "a,\"b\",\"c , x , y\",d",
        vec!["a", "b", "c , x , y", "d"]
    );
    gen_test!(
        test_with_quote_contains_comma2,
        "a,\"b\",\"c , x , y,,\",d",
        vec!["a", "b", "c , x , y,,", "d"]
    );
    gen_test!(
        test_long_text_with_number,
        "\"a\",\"b\",\"long text with number\", 123",
        vec!["a", "b", "long text with number", "123"]
    );
    gen_test!(
        test_long_text_with_trailing_spaces,
        "  \"a\",\"b\",   \"long text with number\"   , 123  ",
        vec!["a", "b", "long text with number", "123"]
    );
    gen_test!(
        test_long_text_with_trailing_spaces_all,
        "  \"a\"  ,    \"b\" ,   \"long text with number\"   , 123  ",
        vec!["a", "b", "long text with number", "123"]
    );
    gen_test!(
        test_long_columns,
        r#"1,22,33,44,abc def,GHI JKL,MNOP,"",2,5555,3333,"ABC DEFG",HIJ KLMNO,"1-2-3",0,X,A B C D E,0,000-000 00:00"#,
        vec![
            "1",
            "22",
            "33",
            "44",
            "abc def",
            "GHI JKL",
            "MNOP",
            "",
            "2",
            "5555",
            "3333",
            "ABC DEFG",
            "HIJ KLMNO",
            "1-2-3",
            "0",
            "X",
            "A B C D E",
            "0",
            "000-000 00:00"
        ]
    );
    gen_test!(
        test_width_20,
        r#"11,22,"YW","5, 6, 7,","","X, Y","","2","ZZZZ","","999901","zzzzz","Ab.","","","","","",,"#,
        vec![
            "11", "22", "YW", "5, 6, 7,", "", "X, Y", "", "2", "ZZZZ", "", "999901", "zzzzz",
            "Ab.", "", "", "", "", "", "", ""
        ]
    );
}
