# diffsync

Client and Server model based upon the idea that clients requests synchronization from the server, 
and the server then supplies the clients with a diff to get a completely synchronized state

## Example

``` Rust

use std::collections::BTreeMap;

use diff::Diff;
use diffsync::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Diff, Debug, Clone, Hash, PartialEq, Default)]
#[diff(attr(#[derive(Serialize, Deserialize)]))]
pub struct Data {
    pub data: BTreeMap<u32, String>,
}

fn main() {
    // clients request updates from the server, after each request,
    // the state should be equal, and it should not be a full update
    // for subsequent requests
    let mut client1 = Client::<Data, u32>::with_id(1337);
    let mut server: Server<Data, u32> = Server::default();

    println!("Creating client with id {}", client1.id());
    println!("Creating 100 values serverside");
    for i in 0..100 {
        server.state.data.insert(i, format!("String {i}"));
    }

    let request = client1.update_request();
    // serialize for some kind of transport, lets try json
    let request = serde_json::to_string(&request).unwrap();

    // #####################
    // Transportation of packet from client to server left as an excersize to the implementer
    // #####################

    let request = serde_json::from_str(&request).unwrap();

    // get the diff between the server and the client
    let clientdiff = server.get_client_diff(request);

    let clientdiff = serde_json::to_string(&clientdiff).unwrap();
    let complete_update_size = clientdiff.len();

    // #####################
    // Transportation of packet from server to client left as an excersize to the implementer
    // #####################

    let clientdiff = serde_json::from_str(&clientdiff).unwrap();

    println!("Client applying update");
    client1.apply_update(clientdiff).unwrap();

    // then if the data changes on the server, the clientdiff will be smaller
    println!("modifying 10/100 entires server state");
    for i in 0..10 {
        server.state.data.insert(i, format!("String {i}"));
    }

    let request2 = client1.update_request();
    // serialize for some kind of transport, lets try json
    let request2 = serde_json::to_string(&request2).unwrap();

    // #####################
    // Transportation of packet from client to server left as an excersize to the implementer
    // #####################

    let request2 = serde_json::from_str(&request2).unwrap();

    // get the diff between the server and the client
    let clientdiff2 = server.get_client_diff(request2);

    let clientdiff2 = serde_json::to_string(&clientdiff2).unwrap();
    let diff_update_size = clientdiff2.len();

    // #####################
    // Transportation of packet from server to client left as an excersize to the implementer
    // #####################
    let clientdiff2 = serde_json::from_str(&clientdiff2).unwrap();
    println!("Client applying update again");
    client1.apply_update(clientdiff2).unwrap();

    println!("\n############Results##############\n");
    println!(
        "Complete update json size (chars): {}",
        complete_update_size
    );
    println!("Partial update json size  (chars): {}", diff_update_size);
}
```
