use nom::{
    bytes::complete::{is_not, tag},
    character::complete::{char, space0},
    sequence::{delimited, tuple},
    IResult,
};

fn non_quote_parse_rest(input: &str) -> IResult<&str, (&str, Vec<&str>)> {
    tuple((is_not(","), csv_line_rest))(input)
}

fn quote_parse_rest(input: &str) -> IResult<&str, (&str, Vec<&str>)> {
    tuple((delimited(char('"'), is_not("\""), char('"')), csv_line_rest))(input)
}

fn csv_line_rest(input: &str) -> IResult<&str, Vec<&str>> {
    if input.is_empty() {
        return Ok((input, vec![]));
    }
    let mut rv = vec![];
    let mut input = input;
    let mut is_quoted = false;
    // clean up space if any
    if let Ok((_input, _)) = space0::<_, ()>(input) {
        input = _input;
    }
    // eat comma if any
    if let Ok((_input, _)) = char::<_, ()>(',')(input) {
        input = _input;
    }
    // clean up space if any
    if let Ok((_input, _)) = space0::<_, ()>(input) {
        input = _input;
    }
    if input.is_empty() {
        return Ok((input, rv));
    }

    loop {
        if let Ok((_input, _)) = char::<_, ()>('"')(input) {
            is_quoted = true;
        }
        if input.is_empty() {
            rv.push("");
            break;
        }
        if is_quoted {
            if let Ok((i, (field, rest))) = quote_parse_rest(input) {
                rv.push(field);
                rv.extend_from_slice(&rest);
                input = i;
            }
            if let Ok((_input, _)) = tag::<_, _, ()>("\"\",")(input) {
                rv.push("");
                input = _input;
                is_quoted = false;
                continue;
            }
        }
        if let Ok((_input, _)) = char::<_, ()>(',')(input) {
            // empty column
            rv.push("");
            input = _input;
            continue;
        }
        if let Ok((i, (field, rest))) = non_quote_parse_rest(input) {
            input = i;
            if input.is_empty() {
                rv.push(field.trim());
                rv.extend_from_slice(&rest);
                break;
            } else {
                rv.push(field);
                rv.extend_from_slice(&rest);
            }
        } else {
            break;
        }
    }
    Ok((input, rv))
}

fn first_part(input: &str) -> IResult<&str, &str> {
    let mut m_input = input;
    // clean up space if any
    if let Ok((_input, _)) = space0::<_, ()>(m_input) {
        m_input = _input;
    }
    if let Ok(_) = char::<_, ()>('"')(m_input) {
        delimited(char('"'), is_not("\""), char('"'))(m_input)
    } else {
        if let Ok((_input, _)) = char::<_, ()>(',')(m_input) {
            return Ok((m_input, ""));
        }

        is_not(",")(m_input)
    }
}

pub fn parse_line(line: &str) -> IResult<&str, Vec<&str>> {
    let (input, (first, _, rest)) = tuple((first_part, space0, csv_line_rest))(line)?;
    let mut rv = vec![first];
    rv.extend_from_slice(&rest);
    Ok((input, rv))
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! gen_test {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let result = parse_line($input).unwrap();
                assert_eq!(result.1.len(), $expected.len());
                assert_eq!(result, ("", $expected));
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
            "11",
            "22",
            "YW",
            "5, 6, 7,",
            "",
            "X, Y",
            "",
            "2",
            "ZZZZ",
            "",
            "999901",
            "zzzzz",
            "Ab.",
            "",
            "",
            "",
            "",
            "",
            "",
            ""
        ]
    );
}
