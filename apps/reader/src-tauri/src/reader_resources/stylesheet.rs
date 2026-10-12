//! Token-aware CSS URL rewriting. Escaped function names are decoded by cssparser.
use super::{component, encode};
use cssparser::{Parser, ParserInput, ToCss, Token};
fn scoped_url(value: &str, pack_id: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty()
        || value.starts_with("//")
        || value.contains(':')
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err("External or active CSS URL".into());
    }
    if let Some(path) = value.strip_prefix('/') {
        let (path, fragment) = path
            .split_once('#')
            .map(|(p, f)| (p, format!("#{f}")))
            .unwrap_or((path, String::new()));
        for part in path.split('/') {
            component(part)?;
        }
        Ok(format!("/pack/{}/{path}{fragment}", encode(pack_id)))
    } else {
        Ok(value.to_owned())
    }
}
fn quoted(value: &str) -> String {
    Token::QuotedString(value.into()).to_css_string()
}
fn walk(
    parser: &mut Parser<'_, '_>,
    pack_id: &str,
    depth: u8,
    string_urls: bool,
) -> Result<String, String> {
    if depth > 32 {
        return Err("CSS nesting limit".into());
    }
    let mut output = String::new();
    let mut import_pending = false;
    while !parser.is_exhausted() {
        let token = parser
            .next_including_whitespace_and_comments()
            .map_err(|_| "Invalid CSS")?
            .clone();
        match token {
            Token::BadUrl(_) | Token::BadString(_) => return Err("Invalid CSS URL".into()),
            Token::AtKeyword(ref name) if name.eq_ignore_ascii_case("import") => {
                import_pending = true;
                output.push_str(&token.to_css_string());
            }
            Token::QuotedString(ref text) if import_pending || string_urls => {
                output.push_str(&quoted(&scoped_url(text, pack_id)?));
                import_pending = false;
            }
            Token::UnquotedUrl(ref text) => {
                output.push_str(&format!("url({})", quoted(&scoped_url(text, pack_id)?)));
                import_pending = false;
            }
            Token::Function(ref name) if name.eq_ignore_ascii_case("url") => {
                let value = parser
                    .parse_nested_block(|p| {
                        let value = p.expect_string_cloned()?.to_string();
                        p.expect_exhausted()?;
                        Ok::<_, cssparser::ParseError<'_, ()>>(value)
                    })
                    .map_err(|_| "Invalid CSS URL")?;
                output.push_str(&format!("url({})", quoted(&scoped_url(&value, pack_id)?)));
                import_pending = false;
            }
            Token::Function(ref name) if name.eq_ignore_ascii_case("expression") => {
                return Err("Active CSS blocked".into())
            }
            Token::Function(_)
            | Token::CurlyBracketBlock
            | Token::SquareBracketBlock
            | Token::ParenthesisBlock => {
                let string_urls = matches!(&token, Token::Function(name) if name.eq_ignore_ascii_case("image-set") || name.eq_ignore_ascii_case("-webkit-image-set"));
                let end = match token {
                    Token::CurlyBracketBlock => '}',
                    Token::SquareBracketBlock => ']',
                    _ => ')',
                };
                output.push_str(&token.to_css_string());
                let inner = parser
                    .parse_nested_block(|p| {
                        walk(p, pack_id, depth + 1, string_urls)
                            .map_err(|e| p.new_custom_error::<String, String>(e))
                    })
                    .map_err(|_| "Invalid CSS block")?;
                output.push_str(&inner);
                output.push(end);
            }
            Token::WhiteSpace(_) | Token::Comment(_) => output.push_str(&token.to_css_string()),
            _ => {
                if import_pending {
                    return Err("Invalid CSS import".into());
                }
                output.push_str(&token.to_css_string());
            }
        }
    }
    if import_pending {
        return Err("Incomplete CSS import".into());
    }
    Ok(output)
}
pub fn rewrite_stylesheet(css: &str, pack_id: &str) -> Result<String, String> {
    if css.len() > 8 * 1024 * 1024 {
        return Err("Stylesheet exceeds budget".into());
    }
    component(&encode(pack_id))?;
    walk(
        &mut Parser::new(&mut ParserInput::new(css)),
        pack_id,
        0,
        false,
    )
}
