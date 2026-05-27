pub(crate) fn parse_query_param(raw: &str) -> Result<(String, String), String> {
    let (key, value) = raw
        .split_once('=')
        .ok_or_else(|| "--query expects key=value".to_owned())?;
    Ok((key.to_owned(), value.to_owned()))
}
