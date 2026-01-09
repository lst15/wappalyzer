
# wappalyzer ( headless_chrome )

This crate is a forked version of https://github.com/garretthunyadi/wappalyzer, however instead of using Reqwests to extract response data, the headless_chrome crate is used. This results in more accurate results, since the rules are ran against rendered versions of the web application.

The forked version also implements support for wappalyzer's "implies" feature, in order to minimize false negatives.

Cargo.toml
```toml
[dependencies]
wappalyzer = { git = "https://github.com/iustin24/wappalyzer" }
```

```rust
let url = Url::parse(&String::from("http://google.com"))?;
let res = wappalyzer::scan(url).await;
println!("{:?}", res);

// Analysis { url: "http://google.com/", result: Ok([Tech { category: "Web Servers",
// name: "Google Web Server", version: None }, Tech { category: "JavaScript Frameworks", name: "ExtJS", version: None }
//, Tech { category: "JavaScript Libraries", name: "List.js", version: None }]) }
```

Or from the executable
```bash
> cargo run cargo run http://google.com/ | jq
{
  "url": "http://google.com/",
  "result": {
    "Ok": [
      {
        "category": "Web Servers",
        "name": "Google Web Server",
        "version": null
      },
      {
        "category": "JavaScript Libraries",
        "name": "List.js",
        "version": null
      },
      {
        "category": "JavaScript Frameworks",
        "name": "ExtJS",
        "version": null
      }
    ]
  }
}```

or given a list of domains in a file:
```bash
> cat urls.list
http://google.com/
http://bbc.com/
...
http://cnn.com/

> cat urls.list | cargo run
{"url":"http://google.com/","result":{"Ok":[{"category":"JavaScript Frameworks","name":"ExtJS","version":null},{"category":"Web Servers","name":"Google Web Server","version":null},{"category":"JavaScript Libraries","name":"List.js","version":null}]}}
{"url":"http://bbc.com/","result":{"Ok":[{"category":"Tag Managers","name":"Google Tag Manager","version":null},{"category":"Analytics","name":"Chartbeat","version":null},{"category":"JavaScript Frameworks","name":"React","version":null},{"category":"Cache Tools","name":"Varnish","version":null},{"category":"Web Servers","name":"Apache","version":null},{"category":"Issue Trackers","name":"Atlassian Jira","version":null},{"category":"Analytics","name":"GrowingIO","version":null},{"category":"JavaScript Libraries","name":"List.js","version":null},{"category":"JavaScript Graphics","name":"Chart.js","version":null},{"category":"Analytics","name":"Optimizely","version":null},{"category":"Analytics","name":"Segment","version":null}]}}
{"url":"http://cnn.com/","result":{"Ok":[{"category":"JavaScript Frameworks","name":"ExtJS","version":null},{"category":"JavaScript Frameworks","name":"Twitter Flight","version":null},{"category":"JavaScript Frameworks","name":"Riot","version":null},{"category":"Advertising Networks","name":"Criteo","version":null},{"category":"Analytics","name":"Chartbeat","version":null},{"category":"Analytics","name":"GoSquared","version":null},{"category":"JavaScript Libraries","name":"Moment.js","version":null},{"category":"Ecommerce","name":"Magento","version":null},{"category":"JavaScript Frameworks","name":"React","version":null},{"category":"Cache Tools","name":"Varnish","version":null},{"category":"Analytics","name":"GrowingIO","version":null},{"category":"JavaScript Libraries","name":"List.js","version":null},{"category":"JavaScript Graphics","name":"Chart.js","version":null},{"category":"Comment Systems","name":"Livefyre","version":null},{"category":"Analytics","name":"Optimizely","version":null},{"category":"Analytics","name":"Segment","version":null}]}}
```
