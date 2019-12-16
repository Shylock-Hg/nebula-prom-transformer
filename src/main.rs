#![feature(proc_macro_hygiene, decl_macro)]
extern crate clap;
#[macro_use]
extern crate log;
extern crate log4rs;
#[macro_use]
extern crate serde;
//#[macro_use]
extern crate prometheus;
use prometheus::Encoder;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
extern crate reqwest;
use std::sync::{Arc, Mutex, RwLock};

const NEBULA_ADDR : &str = "nebula-addr";
const NEBULA_PORT : &str = "nebula-port";
const PORT : &str = "port";

// TODO(shylock) Cache the metrics?
// NOT Cache metrics now
lazy_static! {
    // Protect metrics
    //pub static ref LOCK: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));

    //pub static ref GAUGES: prometheus::GaugeVec = register_gauge_vec!(
        //"Gauges",
        //"Record all gauges from nebula",
        //&["tag"]
    //).unwrap();

    //pub static ref HISTOGRAMS: prometheus::HistogramVec = register_histogram_vec!(
        //"Histograms",
        //"Record all histograms from nebula",
        //&["tag"],
        //prometheus::exponential_buckets(1f64, 5f64, 10).unwrap()
    //).unwrap();

    //static ref METRICS: Arc<RwLock<Metrics>> = Arc::new(RwLock::new(Metrics {
        //gauges: vec![],
        //histograms: vec![],
    //}));

    static ref URL: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

fn main() {
    setup_logging();
    let matches = clap::App::new("nebula-prom-transformer")
        .version("1.0")
        .author("Shylock Hg <shylock.huang@vesoft.com>")
        .about("Transform the raw metrics data from nebula to prometheus defined format")
        .arg(
            clap::Arg::with_name(NEBULA_ADDR)
                .long(NEBULA_ADDR)
                .help("Specify the nebula metric expose address")
                .default_value("localhost")
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name(NEBULA_PORT)
                .long(NEBULA_PORT)
                .help("Specify the nebula metric expose port")
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name(PORT)
                .long(PORT)
                .help("Specify the prot to expose metrics data encoded in prometheus format")
                .default_value("2333")
                .takes_value(true)
        )
        .get_matches();
    let nebula_addr = matches.value_of(NEBULA_ADDR).unwrap();
    let nebula_port = matches.value_of(NEBULA_PORT).unwrap();
    info!("Scrape raw metrics from http://{}:{}/metrics!", nebula_addr, nebula_port);
    let port = matches.value_of(PORT).unwrap().parse::<u16>().unwrap();
    info!("Expose at port http://localhost:{}/metrics!", port);

    let url : String = format!("http://{}:{}/metrics", nebula_addr, nebula_port);
    {
        URL.lock().unwrap().push_str(&url);
        warn!("The url: {}", URL.lock().unwrap());
    }

    // Setup HTTP API
    let config = rocket::config::Config::build(rocket::config::Environment::Staging)
        .address("localhost")
        .port(port)
        .workers(12)
        .unwrap();
    rocket::custom(config)
        .mount("/", routes![hello, get_metrics])
        .launch();
}

#[derive(Deserialize, Serialize)]
#[derive(Debug)]
struct Gauge {
    pub name: String,
    pub value: i64,
    pub labels: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[derive(Debug)]
struct Histogram {
    pub name: String,
    pub value_range: [i64; 2],
    pub buckets: Vec<u64>,
    pub lables: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[derive(Debug)]
struct Metrics {
    gauges: Vec<Gauge>,
    histograms: Vec<Histogram>,
}

impl Metrics {
    fn gauges(&self) -> &Vec<Gauge> {
        return &self.gauges
    }

    fn histograms(&self) -> &Vec<Histogram> {
        return &self.histograms
    }
}

fn setup_logging() {
    use log::LevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::append::file::FileAppender;
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs::config::{Appender, Config, Logger, Root};

    let stdout = ConsoleAppender::builder().build();

    let filelog = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("log/log.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("filelog", Box::new(filelog)))
        .logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
        .logger(Logger::builder()
            .appender("filelog")
            .additive(false)
            .build("app::requests", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();

    log4rs::init_config(config).unwrap();
}

/// Transform the standard metrics to Prometheus format structure
/// Which model by the prometheus 3rd-party library
#[get("/metrics")]
fn get_metrics() -> String {
    let metrics: Metrics = reqwest::get(&*URL.lock().unwrap()).unwrap().json().unwrap();
    return prometheus_format(&metrics);
}


#[get("/")]
fn hello() -> &'static str {
    "The Prometheus metrics exposer for Nebula Graph! Get the metrics from /metrics.\n"
}


// Format Prometheus
fn prometheus_format(m: &Metrics) -> String {
    let reg = prometheus::Registry::new();
    let encoder = prometheus::TextEncoder::new();
    // Gauges
    for g in m.gauges() {
        let gauge_option = prometheus::Opts::new(g.name.clone(), "Record all gauges about nebula".to_string());
        let gauge = prometheus::Gauge::with_opts(gauge_option).unwrap();
        gauge.set(g.value as f64);
        reg.register(Box::new(gauge.clone())).unwrap();
    }
    // Now we expose the gauge computed from histogram!
    // We need to construct histogram with buckets filled!
    //for h in m.histograms() {
        //let histogram_option = prometheus::HistogramOpts::new(h.name.clone(), "Record all histograms about Nebula".to_string());
        //let mut buckets: Vec<f64> = vec![];
        ////for b in &h.buckets {  It's bound
            ////buckets.push(*b as f64);
        ////}
        //let histogram = prometheus::Histogram::with_opts(histogram_option.buckets(buckets)).unwrap();
        //reg.register(Box::new(histogram.clone())).unwrap();
    //}
    let mut buffer = vec![];
    let metrics = reg.gather();
    encoder.encode(&metrics, &mut buffer).unwrap();

    return String::from_utf8(buffer).unwrap();
}
