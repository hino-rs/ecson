#[cfg(test)]
pub mod tests {
    use ecson_network::plugin::EcsonWebSocketPlugin;
    #[test]
    #[should_panic]
    fn inappropriate_address() {
        EcsonWebSocketPlugin::new("127.0.0.1:80800");
    }
}
