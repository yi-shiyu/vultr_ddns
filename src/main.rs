use std::collections::HashMap;
use std::error::Error;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use chrono::{FixedOffset, Local};
use clap::Parser;
use reqwest::{Client, header, StatusCode};
use tokio::time::{sleep, Duration};
use serde::Deserialize;
use serde_json::{Number, Value};
use serde_json::Value::{Number as Nu, String as Str};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 域名头
    #[arg(long)]
    domain_head: String,

    /// 域名体
    #[arg(long)]
    domain_body: String,

    /// 域名设置ttl时常
    #[arg(long, default_value_t = 300)]
    ttl: usize,

    /// vultr key
    #[arg(long)]
    api_key: String,

    /// 查询IP间隔
    #[arg(long, default_value_t = 60)]
    sleep_sec: u64,

    /// 查询IP url
    #[arg(long)]
    check_ip_url: String,

}

struct Vultr {
    args: Args,
    client: Client,
}

impl Vultr {
    fn new(args: Args) -> Self {
        let auth = format!("Bearer {}", &args.api_key);
        let mut auth_value = header::HeaderValue::from_str(auth.as_str()).unwrap();
        auth_value.set_sensitive(true);
        let mut headers = header::HeaderMap::new();
        headers.insert("Authorization", auth_value);
        let client = Client::builder().default_headers(headers).build().unwrap();

        Vultr {
            args,
            client,
        }
    }
    async fn get_record(&self) -> Result<Records, Box<dyn Error>> {
        let response: String = self.client
            .get(format!("https://api.vultr.com/v2/domains/{}/records", &self.args.domain_body))
            .send().await?
            .text().await?;
        let records: Records = serde_json::from_str(response.as_str())?;
        Ok(records)
    }

    async fn create_record(&self, ip: &str) -> Result<(String, StatusCode), Box<dyn Error>> {
        let mut map: HashMap<&str, Value> = HashMap::new();
        map.insert("type", Str("A".to_string()));
        map.insert("name", Str(self.args.domain_head.to_string()));
        map.insert("data", Str(ip.to_string()));
        map.insert("ttl", Nu(Number::from(self.args.ttl)));
        map.insert("priority", Nu(Number::from(0)));

        println!("{:?}", map);
        let response = self.client
            .post(format!("https://api.vultr.com/v2/domains/{}/records", &self.args.domain_body))
            .json(&map)
            .send().await?;

        let code = response.status();
        let res = response.text().await?;
        Ok((res, code))
    }

    async fn update_record(&self, record_id: &str, ip: &str) -> Result<(String, StatusCode), Box<dyn Error>> {
        let mut map: HashMap<&str, Value> = HashMap::new();
        map.insert("name", Str(self.args.domain_head.to_string()));
        map.insert("data", Str(ip.to_string()));
        map.insert("ttl", Nu(Number::from(self.args.ttl)));
        map.insert("priority", Nu(Number::from(0)));

        println!("{:?}", map);
        let response = self.client
            .patch(format!("https://api.vultr.com/v2/domains/{}/records/{}", &self.args.domain_body, record_id))
            .json(&map)
            .send().await?;
        let code = response.status();
        let res = response.text().await?;
        Ok((res, code))
    }
    async fn del_record(&self, record_id: &str) -> Result<(String, StatusCode), Box<dyn Error>> {
        let response = self.client
            .delete(format!("https://api.vultr.com/v2/domains/{}/records/{}", &self.args.domain_body, record_id))
            .send().await?;
        let code = response.status();
        let res = response.text().await?;
        Ok((res, code))
    }
}

#[derive(Deserialize, Debug)]
struct Records {
    records: Vec<Record>,
}

#[derive(Deserialize, Debug)]
struct Record {
    id: String,
    #[serde(rename = "type")]
    r#type: String,
    name: String,
    data: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let vultr = Arc::new(Vultr::new(Args::parse()));
    let ip_cache = Arc::new(Mutex::new(String::new()));

    println!("{}Start", get_time());
    loop {
        let vultr_instance = Arc::clone(&vultr);
        let ip = Arc::clone(&ip_cache);
        tokio::spawn(async move {
            if let Err(e) = do_job(ip, vultr_instance).await {
                println!("{}Error = {}", get_time(), e);
            }
        });
        sleep(Duration::from_secs(vultr.args.sleep_sec)).await;
        // println!("ip:{}", ip_cache.lock().unwrap());
    }
}

async fn do_job(ip: Arc<Mutex<String>>, vultr: Arc<Vultr>) -> Result<(), Box<dyn Error>> {
    let now_ip = get_my_ip(&vultr.args.check_ip_url).await?;

    let ip_cache = ip.lock().unwrap().clone();
    if now_ip == ip_cache {
        return Ok(());
    }
    println!("{}IP change:{} -> {}", get_time(), ip_cache, now_ip);
    let records = vultr.get_record().await?;
    let collect_records: Vec<_> = records.records.iter().filter(|record| record.r#type == "A" && record.name == vultr.args.domain_head).collect();
    if collect_records.is_empty() {
        println!("{}Not Find IP ddns config", get_time());
        let (response, code) = vultr.create_record(&now_ip).await?;
        println!("{}IP create: {}.code: {}", get_time(), response, code);
    } else {
        for (i, v) in collect_records.iter().enumerate() {
            if i == 0 {
                if v.data != now_ip {
                    let (response, code) = vultr.update_record(&v.id, &now_ip).await?;
                    println!("{}IP update: {}.code: {}", get_time(), response, code);
                }
            } else {
                let (response, code) = vultr.del_record(&v.id).await?;
                println!("{}IP del: {}.code: {}", get_time(), response, code);
            }
        }
    }
    *ip.lock().unwrap() = now_ip;
    Ok(())
}

async fn get_my_ip(check_ip_url: &str) -> Result<String, Box<dyn Error>> {
    let local_ip = IpAddr::from_str("0.0.0.0")?;
    let response = Client::builder().local_address(local_ip).build()?.get(check_ip_url).send().await?.text().await?;
    Ok(response)
}

fn get_time() -> String {
    let offset = FixedOffset::east_opt(8 * 60 * 60).unwrap();
    let shanghai_time = Local::now().with_timezone(&offset);
    shanghai_time.format("[%Y-%m-%d %H:%M:%S%.3f]").to_string()
}