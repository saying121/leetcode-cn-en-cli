use lcode::{config::global::glob_leetcode, leetcode::IdSlug, render::*};
use miette::Result;
use pretty_assertions::assert_eq;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn content_img() -> Result<()> {
    let id = 1008;
    let qs = glob_leetcode()
        .await
        .get_qs_detail(IdSlug::Id(id), true)
        .await?;
    println!(
        "{}",
        qs.translated_content
            .as_deref()
            .unwrap()
    );
    let slug = "cnHoX6".to_owned();
    render_qs_to_tty(&qs)?;
    let qs = glob_leetcode()
        .await
        .get_qs_detail(IdSlug::Slug(slug), true)
        .await?;
    println!(
        "{}",
        qs.translated_content
            .as_deref()
            .unwrap()
    );
    render_qs_to_tty(&qs)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn render_md_terminal() -> Result<()> {
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::DEBUG)
    //     .with_test_writer()
    //     .init();

    let id = 108;
    let qs = glob_leetcode()
        .await
        .get_qs_detail(IdSlug::Id(id), true)
        .await?;
    let slug = "ryfUiz".to_owned();
    render_qs_to_tty(&qs)?;
    let qs = glob_leetcode()
        .await
        .get_qs_detail(IdSlug::Slug(slug), true)
        .await?;
    render_qs_to_tty(&qs)?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn render_md_str() -> Result<()> {
    let id = 100_092;
    let qs = glob_leetcode()
        .await
        .get_qs_detail(IdSlug::Id(id), false)
        .await?;

    let a = qs.to_rendered_str(80, 80)?;
    println!("{}", a);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pre() -> Result<()> {
    let id = 654;
    let qs = glob_leetcode()
        .await
        .get_qs_detail(IdSlug::Id(id), false)
        .await?;
    let a = pre_render(&qs);
    println!("{}", a);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn render_md_str1() -> Result<()> {
    let id = 654;
    let qs = glob_leetcode()
        .await
        .get_qs_detail(IdSlug::Id(id), false)
        .await?;

    let a = qs.to_md_str();
    println!("{}", a);

    Ok(())
}

#[test]
fn sub() {
    assert_eq!(&superscript(1_234_567_890), "¹²³⁴⁵⁶⁷⁸⁹⁰");
    assert_eq!(&subscript(1_234_567_890), "₁₂₃₄₅₆₇₈₉₀");
}
