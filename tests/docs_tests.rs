use wiki::docs::parse_markdown;

#[test]
fn parse_markdown_respects_privilege_markers() {
    let doc = "!1\nVisible section\n\n!3\nRestricted section\n";

    let rendered = parse_markdown(doc, 1);

    assert!(rendered.contains("<p>Visible section</p>"));
    assert!(!rendered.contains("Restricted section"));
}

#[test]
fn parse_markdown_returns_placeholder_when_everything_hidden() {
    let doc = "!4\nTop secret\n";

    let rendered = parse_markdown(doc, 1);

    assert_eq!(rendered, "Page requires higher privileges, try logging in.");
}
