use url::Url;

#[tokio::main]
async fn main() {
    let url = Url::parse(&String::from("https://200.150.197.45:443")).expect("ERR");
    let res = wappalyzer::scan(url, Option::from(true)).await;
    println!("{:?}", res);
}