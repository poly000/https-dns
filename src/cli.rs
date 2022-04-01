use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    #[clap(short = 'l', long, default_value = "127.0.0.1")]
    pub local_address: String,

    #[clap(short = 'p', long, default_value = "53")]
    pub local_port: u16,

    #[clap(short = 'u', long, default_value = "1.1.1.1")]
    pub upstream_address: String,

    #[clap(short = 'o', long, default_value = "443")]
    pub upstream_port: u16,
}
