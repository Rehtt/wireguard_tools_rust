mod test {
    #[allow(unused_imports)]
    use crate::Interface;
    #[allow(unused_imports)]
    use crate::CMD;

    #[test]
    fn parse_file_to_wg_struct() {
        let a = Interface::from_file_str(
            "[Interface]
Address = 10.13.13.2/24
PrivateKey = OA2x4YFBii8pgEPvm9Nb7IsBamyfNlTg1lA5m5wyrUo=
ListenPort = 51820
DNS = 8.8.8.8

[Peer]
PublicKey = /A/8ru1OOVcrDMljZcHgxWYH5groyynHxcAdpRca21s=
Endpoint = 116.31.232.209:51820
AllowedIPs = 10.13.13.5/32

[Peer]
PublicKey = SoznFdDKSTgvAIeCMpYHH2y4xvaqJObS3l4AY3XVRzY=
PresharedKey = kguCX9oPV/ACCuaeVOX5OJ9YeLEywsn2oGkCTYN7Fco=
Endpoint = 81.71.149.31:51820
AllowedIPs = 10.13.13.0/24,192.168.31.1/32
PersistentKeepalive = 25",
            "wg0",
        );
        println!("{:#?}", a)
    }

    #[test]
    fn parse_wg() {
        let a = "
interface: wg0
  public key: public_key1
  private key: (hidden)
  listening port: 51820

peer: public_key2
  preshared key: (hidden)
  endpoint: 1.1.1.1:1
  allowed ips: 1.1.2.0/24
  latest handshake: 1 minute, 31 seconds ago
  transfer: 1.18 MiB received, 3.89 MiB sent
  persistent keepalive: every 25 seconds

interface: wg1
  public key: public_key3
  private key: (hidden)
  listening port: 51820

peer: public_key4
  preshared key: (hidden)
  endpoint: 81.71.149.31:51820
  allowed ips: 10.13.13.0/24
  latest handshake: 1 minute, 31 seconds ago
  transfer: 1.18 MiB received, 3.89 MiB sent
  persistent keepalive: every 25 seconds";

        // 解析
        println!("{:#?}", Interface::from_wg(a));
    }

    #[test]
    fn cmd_show() {
        println!("{:#?}", CMD::show_all())
    }
}
