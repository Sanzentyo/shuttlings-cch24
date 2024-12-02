use std::net::{Ipv4Addr, Ipv6Addr};

use axum::extract::Query;
use serde::Deserialize;


// task 1
#[derive(Deserialize)]
pub struct FromKeyQuery {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

pub async fn from_key_calc(from_key_query: Query<FromKeyQuery>) -> String {
    let from = from_key_query.from;
    let key = from_key_query.key;

    Ipv4Addr::new(from.octets()[0].overflowing_add(key.octets()[0]).0,
                    from.octets()[1].overflowing_add(key.octets()[1]).0,
                    from.octets()[2].overflowing_add(key.octets()[2]).0,
                    from.octets()[3].overflowing_add(key.octets()[3]).0).to_string()
                  .to_string()
}

// task 2
#[derive(Deserialize)]
pub struct FromToQuery {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

pub async fn from_to_calc(from_to_query: Query<FromToQuery>) -> String {
    let from = from_to_query.from;
    let to = from_to_query.to;

    Ipv4Addr::new(to.octets()[0].overflowing_sub(from.octets()[0]).0,
                    to.octets()[1].overflowing_sub(from.octets()[1]).0,
                    to.octets()[2].overflowing_sub(from.octets()[2]).0,
                    to.octets()[3].overflowing_sub(from.octets()[3]).0).to_string()
                  .to_string()
}

// task 3
#[derive(Deserialize)]
pub struct FromKeyQueryV6 {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

pub async fn from_key_calc_v6(from_key_query: Query<FromKeyQueryV6>) -> String {
    let from = from_key_query.from;
    let key = from_key_query.key;

    Ipv6Addr::new(
        from.segments()[0] ^ key.segments()[0],
        from.segments()[1] ^ key.segments()[1],
        from.segments()[2] ^ key.segments()[2],
        from.segments()[3] ^ key.segments()[3],
        from.segments()[4] ^ key.segments()[4],
        from.segments()[5] ^ key.segments()[5],
        from.segments()[6] ^ key.segments()[6],
        from.segments()[7] ^ key.segments()[7]
    ).to_string()
}

#[derive(Deserialize)]
pub struct FromToQueryV6 {
    from: Ipv6Addr,
    to: Ipv6Addr,
}

pub async fn from_to_calc_v6(from_to_query: Query<FromToQueryV6>) -> String {
    let from = from_to_query.from;
    let to = from_to_query.to;

    Ipv6Addr::new(
        to.segments()[0] ^ from.segments()[0],
        to.segments()[1] ^ from.segments()[1],
        to.segments()[2] ^ from.segments()[2],
        to.segments()[3] ^ from.segments()[3],
        to.segments()[4] ^ from.segments()[4],
        to.segments()[5] ^ from.segments()[5],
        to.segments()[6] ^ from.segments()[6],
        to.segments()[7] ^ from.segments()[7]
    ).to_string()
}