// use futures::stream::iter;
use tonic::Request;

use common::ss_user_update;
#[allow(unused_imports)]
use common::ss_response;
use common::SsRequest;
use common::SsUserUpdate;
use common::session_client;

fn get_token() -> String {
    String::from("token")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cert = include_str!("../client.pem");
    let key = include_str!("../client.key");
    let id = tonic::transport::Identity::from_pem(cert.as_bytes(),key.as_bytes());
    let s = include_str!("../my_ca.pem");
    let ca = tonic::transport::Certificate::from_pem(s.as_bytes());
    let _tls = tonic::transport::ClientTlsConfig::new().domain_name("localhost").identity(id).ca_certificate(ca);

    println!("Opening channel!");
    let channel = tonic::transport::Channel::from_static("http://localhost:54321")
        // ._tls_config(tls).unwrap()
        .connect()
        .await?;
    let token = get_token();
    let mut client = session_client::SessionClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert(
            "authorization",
            tonic::metadata::MetadataValue::from_str(&token).unwrap(),
        );
        Ok(req)
    });
    // let request = tonic::Request::new(iter(vec![
    //     SayRequest {
    //        name:String::from("anshul")
    //     },
    //     SayRequest {
    //        name:String::from("anshul")
    //     },
    //     SayRequest {
    //        name:String::from("anshul")
    //     },
    // ]));

    // let request = tonic::Request::new(iter(vec![
    //     SayRequest {
    //         name: String::from("anshul"),
    //     },
    //     SayRequest {
    //         name: String::from("rahul"),
    //     },
    //     SayRequest {
    //         name: String::from("vijay"),
    //     },
    // ]));
    println!("Preparing request...");
    let request = tonic::Request::new(SsRequest {
        user_id: 123,
        msg_timestamp: 9878374,
        user_hash: String::from("blah-blah-blah"),
        start_point: Option::from( SsUserUpdate {
            latitude: 0.0,
            longitude: 0.0,
            location_timestamp: 23984794,
            status: ss_user_update::Status::Passive as i32,
        } ),
        opt_is_continuation: Option::default(),
    });

    println!("Trying to open a session...");
    let response = client.open(request).await?.into_inner();

    // while let Some(res) = response.message().await? {
    //     println!("NOTE = {:?}", res);
    // }

    println!("RESPONSE: session ID = {}", response.session_id);

    Ok(())
}
