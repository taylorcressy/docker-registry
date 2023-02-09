use clap::Parser;

#[macro_use]
extern crate lazy_static;
extern crate atty;

mod conf;
mod registry_client;
mod http_client;
mod auth;


#[derive(clap::ValueEnum, Clone)]
enum Command {
    LIST,
    TAGS,
    MANIFEST,
    DIGEST,
    DELETE,
}

#[derive(Parser)]
#[clap(
    name = "Docker Registry V2",
    version = "1.0",
    author = "Taylor Cressy",
    about = "A CLI wrapper around the docker registry V2 API"
)]
// #[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(index = 2)]
    registry: String,

    #[arg(value_enum, index = 1)]
    command: Command,

    #[arg(short, long)]
    username: Option<String>,

    #[arg(short, long)]
    password: Option<String>,

    // TODO: Change this to a boolean and name secure instead of having to specify the protocol explicitly
    //  Set default to true
    #[arg(short = 's', long = "proto")]
    proto: Option<String>,

    #[arg(short, long)]
    image: Option<String>,

    #[arg(short, long)]
    tag: Option<String>,

    #[arg(short, long)]
    digest: Option<String>,

    #[arg(long)]
    conf: Option<String>,

    #[arg(short, long, default_missing_value = "true", value_parser, default_value = "false")]   
    verbose: bool,

}

#[tokio::main]
async fn main() {
    let args: CliArgs = CliArgs::parse();

    if args.verbose {
        std::env::set_var("DOCKER_REG_VERBOSE", "true");
    }

    let app_conf = match conf::load_conf(args.conf) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    };

    let auth = match auth::derive_credentials_through_chain(args.username, args.password, app_conf.clone()) {
        Ok(v) => v,
        Err(e) =>  { 
            println!("{}", e);
            std::process::exit(1);
        },
    };

    let command_context = registry_client::CommandContext {
        username: &auth.username,
        password: &auth.password,
        proto: args.proto,
        image_name: args.image,
        tag: args.tag,
        digest: args.digest,
    };

    match registry_client::is_v2_supported(&args.registry, &command_context).await {
        Ok(_) => (),
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        },
    }

    let _ = match args.command {
        Command::LIST => match registry_client::list_images(args.registry, command_context).await {
            Ok(value) => for i in value {
                println!("{}", i);
            },
            Err(error) => println!("{}", error),
        },
        Command::TAGS => match registry_client::get_image_tags(args.registry, command_context).await {
            Ok(value) => for i in value { println!("{}", i); },
            Err(error) => println!("{}", error),
        },
        Command::MANIFEST => match registry_client::get_image_manifest(args.registry, command_context).await {
            Ok(value) => println!("{}", value),
            Err(error) => println!("{}", error),
        },
        Command::DIGEST => match registry_client::get_image_digest(args.registry, command_context).await {
            Ok(value) => println!("{}", value),
            Err(err) => println!("{}", err),
        },
        Command::DELETE => match registry_client::delete_image(args.registry, command_context).await {
            Ok(_) => println!("Okay"),
            Err(err) => println!("{}", err),
        }
    };
}
