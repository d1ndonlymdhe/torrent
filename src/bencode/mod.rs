use std::collections::HashMap;
use crate::str_utils::{index_of, sub_str};

type BString = String;
type BInt = i64;
type BDict = HashMap<String, Bencode>;
type BList = Vec<Bencode>;

#[derive(Debug, PartialEq)]
pub enum Bencode {
    Str(BString),
    Int(BInt),
    List(BList),
    Dict(BDict),
    End,
}

impl Bencode {
    fn new_str(str: impl Into<String>) -> Self {
        Bencode::Str(str.into())
    }
}

enum BencodeTypes {
    Str,
    Int,
    List,
    Dict,
    End,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct ParseResult<T> {
    data: T,
    len: usize,
}
impl<T> ParseResult<T> {
    fn new(data: T, len: usize) -> Self
    {
        ParseResult {
            data,
            len,
        }
    }
}

pub fn parse_bencode(line: impl Into<String>) -> Result<ParseResult<Bencode>, ()>
{
    let line = line.into();
    let ben_type = get_type(&line);
    match ben_type {
        BencodeTypes::Str => {
            let res = parse_string(&line);
            if let Ok(res) = res {
                let ParseResult { data, len } = res;
                Ok(ParseResult::new(Bencode::Str(data), len))
            } else {
                panic!("Invalid String {}", line)
            }
        }
        BencodeTypes::Int => {
            let res = parse_int(&line);
            if let Ok(res) = res {
                let ParseResult { data, len } = res;
                Ok(ParseResult::new(Bencode::Int(data), len))
            } else {
                Err(())
            }
        }
        BencodeTypes::List => {
            let res = parse_list(&line);
            if let Ok(res) = res {
                let ParseResult { data, len } = res;
                Ok(ParseResult::new(Bencode::List(data), len))
            } else {
                Err(())
            }
        }
        BencodeTypes::Dict => {
            let res = parse_dict(&line);
            if let Ok(res) = res {
                let ParseResult { data, len } = res;
                Ok(ParseResult::new(Bencode::Dict(data), len))
            } else {
                Err(())
            }
        }
        BencodeTypes::End => {
            Ok(ParseResult::new(Bencode::End, 1))
        }
    }
}

fn get_type(line: impl Into<String>) -> BencodeTypes {
    let line = line.into();
    let first_char = sub_str(line, 0, 1);
    match first_char.as_str() {
        "i" => BencodeTypes::Int,
        "d" => BencodeTypes::Dict,
        "l" => BencodeTypes::List,
        "e" => BencodeTypes::End,
        "" => BencodeTypes::End,
        _ => BencodeTypes::Str,
    }
}
fn parse_string(line: impl Into<String>) -> Result<ParseResult<BString>, ()> {
    let line = line.into();
    let separator_idx = index_of(&line, ':');
    match separator_idx {
        Ok(separator_idx) => {
            let len = sub_str(&line, 0, separator_idx);
            let len = len.parse::<usize>().unwrap_or_else(|_| { panic!("Invalid string") });
            let string = sub_str(&line, separator_idx + 1, len);
            Ok(ParseResult::new(string, separator_idx + 1 + len))
        }
        Err(_) => {
            Err(())
        }
    }
}
fn parse_int(line: impl Into<String>) -> Result<ParseResult<BInt>, ()> {
    let line = line.into();
    let first_char = sub_str(&line, 0, 1);
    if first_char == "i" {
        let index_of_end = index_of(&line, 'e');
        if let Ok(index_of_end) = index_of_end {
            let num = sub_str(&line, 1, index_of_end - 1).parse().unwrap_or_else(|_| { panic!("Invalid Integer") });
            return Ok(ParseResult::new(num, index_of_end + 1));
        }
    }
    Err(())
}

fn parse_list(line: impl Into<String>) -> Result<ParseResult<BList>, ()> {
    let line = line.into();
    let first_char = sub_str(&line, 0, 1);
    if first_char == "l" {
        let mut new_line = sub_str(&line, 1, line.len());
        let mut ret_vec = Vec::new();
        let mut total_parsed = 1;
        loop {
            let bencode = parse_bencode(&new_line);
            if let Ok(res) = bencode {
                let ParseResult { data, len } = res;
                if Bencode::End == data {
                    total_parsed += len;
                    break;
                } else {
                    total_parsed += len;
                    new_line = sub_str(&new_line, len, new_line.len());
                    ret_vec.push(data)
                }
            }
        }
        return Ok(ParseResult::new(ret_vec, total_parsed));
    }
    Err(())
}
fn parse_dict(line: impl Into<String>) -> Result<ParseResult<BDict>, ()> {
    let line = line.into();
    let first_char = sub_str(&line, 0, 1);
    if first_char == "d" {
        let mut new_line = sub_str(&line, 1, line.len());
        let mut ret_map = HashMap::new();
        let mut total_parsed = 1;
        loop {
            let bencode_str = parse_bencode(&new_line);
            if let Ok(res) = bencode_str {
                let ParseResult { data, len } = res;
                total_parsed += len;
                if let Bencode::Str(map_key) = data {
                    new_line = sub_str(&new_line, len, new_line.len());
                    let benccode_value = parse_bencode(&new_line);
                    if let Ok(res) = benccode_value {
                        let ParseResult { data, len } = res;
                        total_parsed += len;
                        ret_map.insert(map_key, data);
                        new_line = sub_str(&new_line, len, new_line.len())
                    } else {
                        panic!("Invalid bencode")
                    }
                } else if data == Bencode::End {
                    break;
                } else {
                    panic!("Invalid bencode")
                }
            }
        }
        return Ok(ParseResult::new(ret_map, total_parsed));
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use super::*;

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string("4:abcd"), Ok(ParseResult::new(String::from("abcd"), 6)));
        assert_eq!(parse_string("0:"), Ok(ParseResult::new(String::from(""), 2)))
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(parse_int("i123e"), Ok(ParseResult::new(123, 5)))
    }

    #[test]
    fn test_list() {
        let test_str = String::from("l4:spam4:eggsi-234el4:spam4:eggsi-234e4:mdheee");
        let lhs = parse_list(&test_str);
        let rhs = Ok(
            ParseResult::new(
                vec![
                    Bencode::new_str("spam"),
                    Bencode::new_str("eggs"),
                    Bencode::Int(-234),
                    Bencode::List(vec![
                        Bencode::new_str("spam"),
                        Bencode::new_str("eggs"),
                        Bencode::Int(-234),
                        Bencode::new_str("mdhe")
                    ]
                    )
                ]
                , test_str.len()));
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn test_dict() {
        let test_str = String::from("d4:listli12e3:zln6:whatupd1:k1:vee4:mdhe4:here3:numi-234ee");
        let lhs = parse_dict(&test_str);

        let mut map = HashMap::new();
        let mut inner_map = HashMap::new();
        inner_map.insert(String::from("k"), Bencode::new_str("v"));
        map.insert(String::from("list"), Bencode::List(
            vec![
                Bencode::Int(12),
                Bencode::new_str("zln"),
                Bencode::new_str("whatup"),
                Bencode::Dict(inner_map)
            ]
        ));
        map.insert(String::from("mdhe"), Bencode::new_str("here"));
        map.insert(String::from("num"), Bencode::Int(-234));
        let rhs = Ok(ParseResult::new(map, test_str.len()));
        assert_eq!(lhs, rhs);
    }
    #[test]
    fn test() {
        let content = read_to_string("../../test.torrent").unwrap();
        let content2 = r#########"d8:announce49:udp://tracker.leechers-paradise.org:6969/announce13:announce-listll49:udp://tracker.leechers-paradise.org:6969/announceel48:udp://tracker.internetwarriors.net:1337/announceel42:udp://tracker.opentrackr.org:1337/announceel43:udp://tracker.coppersurfer.tk:6969/announceel42:udp://tracker.pirateparty.gr:6969/announceel30:udp://9.rarbg.to:2730/announceel30:udp://9.rarbg.to:2710/announceel38:udp://bt.xxx-tracker.com:2710/announceel38:udp://tracker.cyberia.is:6969/announceel42:udp://retracker.lanta-net.ru:2710/announceel30:udp://9.rarbg.to:2770/announceel30:udp://9.rarbg.me:2730/announceel29:udp://eddie4.nl:6969/announceel36:udp://tracker.mg64.net:6969/announceel35:udp://open.demonii.si:1337/announceel38:udp://tracker.zer0day.to:1337/announceel40:udp://tracker.tiny-vps.com:6969/announceel39:udp://ipv6.tracker.harry.lu:80/announceel30:udp://9.rarbg.me:2740/announceel30:udp://9.rarbg.me:2770/announceel42:udp://denis.stalker.upeer.me:6969/announceel39:udp://tracker.port443.xyz:6969/announceel38:udp://tracker.moeking.me:6969/announceel37:udp://exodus.desync.com:6969/announceel30:udp://9.rarbg.to:2740/announceel30:udp://9.rarbg.to:2720/announceel39:udp://tracker.justseed.it:1337/announceel41:udp://tracker.torrent.eu.org:451/announceel39:udp://ipv4.tracker.harry.lu:80/announceel44:udp://tracker.open-internet.nl:6969/announceel36:udp://torrentclub.tech:6969/announceel33:udp://open.stealth.si:80/announceel35:http://tracker.tfile.co:80/announceel36:udp://open.demonii.com:1337/announceel32:udp://explodie.org:6969/announceel35:udp://p4p.arenabg.com:1337/announceel36:udp://tracker.dler.org:6969/announceel33:udp://movies.zsw.ca:6969/announceel46:udp://tracker.openbittorrent.com:6969/announceel41:udp://uploads.gamecoast.net:6969/announceel41:udp://tracker1.bt.moack.co.kr:80/announceel41:udp://opentracker.i2p.rocks:6969/announceel35:udp://bt1.archive.org:6969/announceel42:udp://tracker.swateam.org.uk:2710/announceel45:http://tracker.openbittorrent.com:80/announceel43:http://tracker.opentrackr.org:1337/announceel36:https://tracker1.520.jp:443/announceel44:https://tracker.tamersunion.org:443/announceel42:https://tracker.imgoingto.icu:443/announceel36:http://nyaa.tracker.wf:7777/announceel35:udp://tracker2.dler.org:80/announceel38:udp://tracker.theoks.net:6969/announceel35:udp://tracker.dump.cl:6969/announceel37:udp://tracker.bittor.pw:1337/announceel44:udp://tracker.4.babico.name.tr:3131/announceel33:udp://sanincode.com:6969/announceel50:udp://retracker01-msk-virt.corbina.net:80/announceel40:udp://private.anonseed.com:6969/announceel40:udp://open.free-tracker.ga:6969/announceel38:udp://isk.richardsw.club:6969/announceel32:udp://htz3.noho.st:6969/announceel29:udp://epider.me:6969/announceel36:udp://bt.ktrackers.com:6666/announceel27:udp://acxx.de:6969/announceel29:udp://aarsen.me:6969/announceel38:udp://6ahddutb1ucc3cp.ru:6969/announceel31:udp://yahor.of.by:6969/announceel34:udp://v2.iperson.xyz:6969/announceel40:udp://tracker1.myporn.club:9337/announceel40:udp://tracker.therarbg.com:6969/announceel33:udp://tracker.qu.ax:6969/announceel45:udp://tracker.publictracker.xyz:6969/announceel38:udp://tracker.netmap.top:6969/announceel38:udp://tracker.farted.net:6969/announceel41:udp://tracker.cubonegro.lol:6969/announceel35:udp://tracker.ccp.ovh:6969/announceel37:udp://tracker.0x7c0.com:6969/announceel35:udp://thouvenin.cloud:6969/announceel40:udp://thinking.duckdns.org:6969/announceel33:udp://tamas3.ynh.fr:6969/announceel29:udp://ryjer.com:6969/announceel41:udp://run.publictracker.xyz:6969/announceel43:udp://run-2.publictracker.xyz:6969/announceel46:udp://public.tracker.vraphim.com:6969/announceel44:udp://public.publictracker.xyz:6969/announceel37:udp://public-tracker.cf:6969/announceel34:udp://opentracker.io:6969/announceel31:udp://open.u-p.pw:6969/announceel33:udp://open.dstud.io:6969/announceel35:udp://oh.fuuuuuck.com:6969/announceel32:udp://new-line.net:6969/announceel35:udp://moonburrow.club:6969/announceel34:udp://mail.segso.net:6969/announceel42:udp://free.publictracker.xyz:6969/announceel30:udp://carr.codes:6969/announceel35:udp://bt2.archive.org:6969/announceel35:udp://6.pocketnet.app:6969/announceel36:udp://1c.premierzal.ru:6969/announceel36:udp://tracker.t-rb.org:6969/announceel37:udp://tracker.srv00.com:6969/announceel42:udp://tracker.artixlinux.org:6969/announceel40:udp://tracker-udp.gbitt.info:80/announceel43:udp://torrents.artixlinux.org:6969/announceel28:udp://psyco.fr:6969/announceel39:udp://mail.artixlinux.org:6969/announceel29:udp://lloria.fr:6969/announceel38:udp://fh2.cmp-gaming.com:6969/announceel30:udp://concen.org:6969/announceel32:udp://boysbitte.be:6969/announceel30:udp://aegir.sexy:6969/announceee7:comment28:dynamic metainfo from client10:created by10:go.torrent13:creation datei1642106107e4:infod5:filesld6:lengthi3548941307e4:pathl75:Pushpa The Rise (2021) 1080p AMZN WEBRip x265 Hindi DDP5.1 ESub - SP3LL.mkveed6:lengthi21688054e4:pathl10:Sample.mkveed6:lengthi101692e4:pathl26:Pushpa The Rise (2021).jpgeee4:name71:Pushpa The Rise (2021) 1080p AMZN WEBRip x265 Hindi DDP5.1 ESub - SP3LL12:piece lengthi4194304e6:pieces17040:zµzßÚÖ¿º÷/D<ù¶¾õ¾^sÅ»Éém)kW=[ü5^vX†„/“ª×B¨†©õ×ò?MµðØ¸Çô”	)l¨81r…|·ÝØ‚¨çžçIë'êÃ˜Úà9oÞœ;‰òûsÅ.rQéµ(š:uÀqè¤W½¬™ÓÁ>=š£weî�÷ÆL°ƒNnYÌ¿&¶È“êxçLå{á(TÒ™Öþƒ`L‘_A’=Ú€ÏŸ6º½{SIJ
vÝ"ÆÇÉV"e‹%Ë¡+—<Ô£Ð<©nÕ¬ÖL?âbŸÑ~ÝúOýú qº),Ü?8¨½vÇÂ@pÜÅo”_{ÝduŠn„çÝa0&™B»çf(¶kïîMùÌn˜~ÿ½aû¹)I§R-<H4”‰xoö±‘¢\¤V§ê‡Íê¿>Oá(2uR$ÁÆñR,ôî…Nç¨ÙÁ]ìV$Ôjà±öqVjl68«þof-.�ò-pÙŸ®q7KÙéè«ÚxÍæÐ!j4:–2W¿jSA?RFÅÔÌ¾à)b|¼òoüJ6wðÚ”°n…õÅÉ†ðº¾$ê
`³¶±¿ ¾#Š‡NÎ‚rù#¹w�Tžl×rLÚ9Wz\­ûNo2� %Ä'‘ÃJ#ŽÚ5à[ÿB3Ú9†z·roëBp}!˜sh®S)˜T^+EM?Xš;C’­ìÃá²pnüutö$ºÂ.øþ´xmŸÕ�Gè€=î[ä£=t
¦Nåæ™¦<U{#7Ù
¸¥…N{ÿ´r+‘}‹O£/)°4B¹+ÜFÜO8	~ÍA)r›Bu=cNFÑ+/<€·ˆÃß-tù
èl«üðñ$$ç³û5uÈŒkä	æÔü2Cô´ŒçÛs‰ÿaC˜ynü[Zæ¢‡Ï1‘[jR4ÊÎ¯Þ¤î>–À+…“I®º"Œîo"¸ì³mã±4pÎKSájèÉ4Ãc¤çMž4>ÙM ì!^ÅÂp(p{É]Qe¬;F‰¦BÐA
M¶ý§9|ÉŽ‚«/§ŠRÚ˜HXÕŒnT«ã“ñùOùái€Û+»]ZYVAïÐ¶?ù+©–«Fê{—o¾¡þ@sõ‡]àæW¬8ÎfTŒëï5·/÷R¯¬øµñ°j«Ÿ´6-ÓÛLGJàÐµZñÛ—PÌ˜ñB¸?/C)&±ËN¥jäT5nO˜®Õ(Ê½T~YÌiëÓMHð‰n¯®UÆœý©ZÁ>éá
vEH˜(ÁƒÆ¬^Ü¢E[¦X×ãkðvHJsOR´ý¤o0u›+ÓnÈ
¥Pð:±ÔcK'¢í¹éýÎ9Ò.¸Ü~|¸Ä‹·½û£ zk},"Ñ8=O¢ÐÚïƒønçÌ+¥l‡ÖÿÅ %kÂþ°£ßþŸåÀúê½gENá‹½ÍgûmÐ0ŠÙ„~];„€NÙœ»&ù·Â¾‘?–;¼"s}Ù†ÿ3êù‚bùþxxJ.³õYFmÚ™r}J×qùšIelêÐe_ý
¢Ä„Óè«FEÄ‚ãnW"¦BaVí	GÓô¿Ã+4´=ÀÅ,™&½ê@ì8¹ç&gØp°”)+Htäƒg>—yØ×=¤*•~õªÚƒx®ê¯Ž-àE &<ú’Gô¼DôèÛ•!}ýx®uq¶´OlØA¾£oLTp²Í’‰5×¥úG{ÅëÁüÛÒ/„¿$²ÝâÅû‹õ^ï:{F¨— TŠmJùÿ#_ŽƒSËçVé£¥cÐ¼d¶»õKÍŸë¶ÊJÌm„pï«K«ßT"ÕTx}íž’6õ¹X)«Õ¶“ÌWz>fVx.Md“GG÷¸
d Òò3810S’B‰Ô&ßÖÙlÊßÃÆŠÌÍ×µ~aÛ“sGØ¹¢ tÁ´çùxŸZv†ÅŠÒ3KÀ¹ð@?.v¼oñUÀŠV-QYùÁ“/¢
Ù¾|Žì—e‚8„’W§¡9m5=ê»ë»ÅU·	|²1iô‡ÿõ4º±Ö˜Åváâ­§al.jœ5%?úªÊmƒÌò.õè*†îrN`ìõHs‰ÞR8þ#§«ZNPªî[QøŸMPLÇøÃfvv|zÖud}Qpú_r*g=¾Àúºxš¼7h„Noæ	È“*º¯¶ñôc›šðÓE@¿Ê_Cÿø˜œ¬³\’§™Üã£øV5±ó@#Ó70k‰8¤ó2}ŸðÖ0w ù€½ä…´(Nšüæº¸§5„îVãá«Õ
G‘ýÊó‡’:.Ûšl€`½»bã5`f ~âŠH K•ß÷žþOüfý†ËdP×fB² § kº0nùûN}­rÏgåpŒe×¯®Ô‘.cGã›}hTþ£ƒÏÑ;X·Éû¾Á.]#¼iV¯*òKÀ±i„Ã_U³Ï)…ÏÆ5¡Fï–5CˆøÇú[ùO	ä³‘¥Ø;j=²¿7†ØýäYèôu&¶Mxk“]Z%õ®ÑÚòóSÊ2îüšO¼ÚÁ�[›û6A¶éŠô½èYQ(êw»éá3$ƒYÄ‚Õ¸è2º€ç¼Æ`‘`sMcöñTÙ}·ÆŒÆ«�¾*#™àVsìRÄª›îŠ(‡t[
Ÿ<`Ò$d«¢äM¾ïS+±b`bi§g'fIªî{+:S¦•§Ò§ÑsFÌ‡+Ÿf#‡Wr¶ŽÄ zPç¡+šƒs¤¸§ïöõãÅÝ’›V SþÌ�5t]MÇÍéOòWmÝx‰~…)´À’a °^ÍÞô£²vbÈ†å”ÞogÁ@/w–’©Œ#D¹ûAvzgŽà†T¤¬P2N°5ù>69Õ˜zÄæ¡±”û?ð$F×(èbÑø‡<¼å§j—®,ýyEsöØD$Ì±:–>úodÇWû¡º~¿Î|áÛu4Ð4*©ìÜÈþQ5þ¸”ãU¹
Òý¨n°�ž<Sl$€5Eœ48•uG,e8–[1J˜;{ÌD—4ö¿`¡6@O%ø´Ó€FÇ.ë<3*½*“cC¦D€ÓŸD•êÐ—ÿßm4Øa¡0sòŸç#˜žÌ0Ñ´C\íE©ìÎ‚ ðžvaÍ‚†«Í«‰Ûãa¤b'ãY˜�ó«¸×±^Þ|GoãSQí¤ã7rR
�we·ŠÔ?]"ÞwŽ(7o&sú†”·ÚÑþÓD)&Sö]—G1aÖêJøhhìöD/´LyâjrG†±	¾GÄ¶6Ü‰ðnxŠÔ7ÂÃ:¿Nš¼¿ÎAx¨ä„“ãâ8ª¹Ý2uf”?–¦P#ižœ´¯g}õ|ÑûÕ7´#ÍÌj•¥¾¬ßÆgÙ8öÁ“3¬©ìNùKÂg¾Ät¨!œ†Š‚xhVh²Ø&3fÂå Á†½•X, ²'¤Àt“‚ÜHŽb6Ã´¡nA'´Æ!™á’ì‡ÐhóžŠ£^”ùQhö
8I¬`"rº[xä½Ž­ þPI!·¬ñûIU#igYaiÃU?üo6ôþT^¾¿^cÈ‰Gûl]ÖoZcQ^‰5nášGa
:Ø�Ï‡ÈµIý>îê2Ý‚ò“ÚÔ@ž%Ç9‹˜±13Ûÿ™ÐÀi³’¦ëRƒù’²ƒþï©Eõq¹™uìÌ»’tCì³Ûúrò°Â{,!mzù:áŽº	&èÔ|G–}�¡‚Ir¨	Ì1ªÄ2ÑÌ^ŒÚæãFéf>9ÝÏï!9¶Ûh«ËsbÇ¥ù
¶Á„
cSmÉ—X©
Î
L°HÁ§Åˆ Ddžpw‡­é™rKû•Ø¸ÖÜ4[~4n
Ûl&X© â´K[Þno�›IIRü
Ôß&Èü]&†áñŸ8ñJÓ¬{ÈZŽ²
Ö²oÍZ½Ü4zø¤$Â¸RóÉ=óp†¢Mï†7›lCHà¢Ãb=‰ÿî'¤	r·_P`2øÕ¯·Æh1‚Á†ÞAOãÝ
oåP?ÑCLÖ<ã.1>Ý:õ†¦²lˆf9±úqqñ /E‡Cè&•05 Â”v2ø®×™%éýB’|
ç18œ=À#¢tX—RxßRÓßãÔCæ­¤‚ÁvQüÐ-WW-V;=Qhpà5ˆ?ÑßrnjíúqÎöè‚~
[½QáÂø¼+É*@|r+C€©‚G5–„*\�C¬äœÖ&s
èàmi#³¤{ÿ!tt÷l´`%RÛµú
_Ï9^\ÊÈ§¼êAb|ìí]YšLÀvØù‹.5©Ú’WY*v¨5¬”�•‚ÐÁ´äò´m£Xx(Š=N;\<¿i°´ºýÝ%höCÃð“Õp/{7Z\²æµz-4è÷•Õæmsô~7×„ç4ß7­ü¬ÆÂ5<ÅÎ¬"à°ã„úÌDët3´¡†•Bo•CÓRb"â�±½~^Z!„›ö0Ëçq[ /ñ	ŽEusÐÈºŠ¾Ö]‚ âö1ž—£lÐ‰Šæb2ÑM¼¸KÀ¨IªE0·OreÕ»Ž*¾ûÚCžŠÌÁïñ¹YõÎŒ]>Ô3H‚> !ôH¤ÔÞ
ñ¨¨N¾zoC)9ÚïÝ›ž0º 4<âÁ#d±^ãq9ƒXöÀQâÄb
õÐË—óX{ÂeUZÊÂ3ÀQVJ³¶ÜÜ™Ò,§bdp€bÀ£:ÏæÛ®§Ê1¨‡¹G™’¸Ð(ùÂ’ü
&EÂ½X;Ä§ef¦¨L–šdEÎEOo)#?ÑH«0I?‡3 ‹ñ£‘Œ_L±âU·7™Ã˜7PÞjáä•%¦Ø½¤[L–¸d²œØ•kZ”Þr©-øîdQ±ãjIW«Ñ0´7â`jÑÅÚ_–LÄàåDægB'Šì6Eôÿ™‡ýÊ¢…I­#ãï|JÓ+JØëP€â«^ð(4´,2:ÿb×Üeí.½ºË«‰¥ 
'¶µÕéJD6~P¶Ì®¡ë±A¼ú1–©´aúW™(„ÑÕê{\‘)ìÊºdóweëU`:†>ŒùkË§}VJm‹™Nk¼úúöp¬”‹ŸÇ„¹÷Ð2Á¼à÷#é4=IÕ½7š"1Þ›ÌBkì6˜€´l{}¢€l.bÜ§)§eu‚Ö1À$ô”Ž‹F¦Ì�¡ré›9Ý'Å®¹Œ0†;‚w8ŽQðóaÕ»åÔš»ž¿ÄÍ¡·†—D\c;úÖ˜ñƒ®äºÕ†iŸH:Hø´ëœ&°¶“4'Šu¬œG\A¹ÎÿuÅF£c™lF’ü­&áýÑÅbOcz”Ðk[–9c2HÜ<Qü[¬Îû®Ø(ð¹Ú%=Ñ€›˜|wRøZs©ëÌ…¿?Ž›·9Ð08o¹|ÉØÅmÍŸôPµÂ¨+Ü.šÉzãcì™I%è)F$1¸µ–=QŸ¸àâ™ÜŸ;ƒßÆH2lŽ^Ëj6âŠÚÿr…X÷}¹'-º±‘CVÇ¯DªÑM"gÜ­ˆB$¥RŠ¯M}›öºÍ_ÃKl8xçP‰?ßZÁ±É–,ÿ‹£7’j;b¿ªë=æaø LÆ2†Nð8*³¸Ë&"SEv–Ò´Ü÷ŸL?NŸþ<ì”9Æ{Šž‘¿Áœµ©0Ì¤Ñ‡TÕ
çgÙZpÐ¾* ‰}kÓ¦†,ù£#È†­«'Hˆ¨IÌÑÅ6Å—¡º÷µ“XâJI~VÐüjspþ‘ó¶ÂñòP{Q‚ùÔM.æ€m'¼"ôÏ€\N÷ÞíSw3µ,{Ïî¦vw@ÿ
ðÛ‰¹‹hðƒ‡›Ïèa{ÈôNÑÅn‡ºž:«îqš�tñ;Cdvhì_®tÀ:¥äU†ìù‹ÇZ÷b–ÝÊ¤> °´Ð¯Äµ—4
‹K¡¨¥¡m@¨ÕRýZ©OÛj	ïjfù±:_f-7«¡ÃŒÜ¾IÖÄ_´häðùm*!h3»€ñO1‰›®Í<FÝÂ©ÍB1Ð{¤o¾±&‰/í*@ëÊÍýêë¿3‘rîxã´@þ•Ðæ¹qX±’°®‰*¬k®hñ*âØÑhõgÃc2É
ÝìÊ9m£ðB±t°"ŠZ7­V¯nÃJÇçd¥v0ðj¾%¯«Gøªúù‰²¶XÆPÌN9*^ÝKw ^xôN‡…IÒê¤J^¶ºÓéÐú­!Må€lÿ-w½ô[´H'ì]Œ!­–>.¥6`Á²õJÝ£Zô¥7;Uûü"šWnË+6¹‰Žf¡’¼úmîßÄ·ÚX‚dI0¿K„ÇŸ“´A‹!Žòöcë{+ýÁTô}äD¡ßÆðDÚ/.Ã—èØ¢qºzl¸ãmèØ»­Á0õŒdq=úMné"ƒÞ‘\¾F²`—"åÝëá’rÚHÙ+Ç£ÊE×ÕéŒ2™Ó`m¡‚Ù„×±Þ&EHÀ<ª¯K$"J6l¾“ŒæÞ\‡"Ì£äª±Ü÷AÂÏmi½7SQVWk³'Ü%Ðƒ/½\¡ÑÁÓ!l±ß¼°nâ-Ë¦«ˆZ` €¤£c'd:Êos<FJt½Žëàz½!È½M²ãˆqRŠ*ÓN‡ý0Å(_ežëÙ±?k‰JÅ&GplÜl²ºtQ:(q	TÉ“ï0<Â¶ñ‡¥å]LMí+E D?qåXl› ÃÒ×Ôóô^¶.Mõ¶#kG`e¨Ý‘q!ð˜w ¸Çú7rŒ�Õg¸À:PliàÊÂ”ó&OïSãÆ,ï$!Ê‚@Êh¸u†¶Tð?_€ã»"PŒì/ˆòx&íÇ1-/Y4pfäiGû>=ãÀºbÏU§À˜È>ß±ýÑH²†
«„Žº3¹„œf±Ä×®Ôa‰8{ÀðßÎ„´rÃà_XàD7,ð¬?4ÀÂšÐÅË~yF‰Ú*P\mÊ™LülªÕ½škiz›«Ja\“û£È™i³J	K’^ß*j©þYâÃ3„´æŠ	«^\ÿ5çs!ÒHyÁÿ`"7ÕñÝŒ.$âû(¶ZŠê ËE’[z:Ì—U‹­858‡49ÞÏ‰wH;yävC6o�J=56ÅzIâV�ü0­Ö@Ñ1	"áC’6U-ïŠÏ˜A�¶öˆ¯ï!¿Â”íÀNÜÅ@ÎÑo@"^V:h];	±Só]|ÂÉ­À“FÓðIùH"±(_íGŸò•ôSfAÕ?ßëKÞ½7äÅU†ˆÝ5’ºÎÝ­0|’§2hUóÁ[—ôÏ1kjBbŠÑñquÏœÇ“L®Å¾u•à•0¦pÁ,iæøó#ÉÛi…ª¢ÞßMz"Î¹·+²S¶ÇcìÝVÅdŽ& 6þ~ŠW¢›ý©›C]–>0c!2.¶åW’¥Vüµ=lQ…&+ë4(zFß4×	9ì¢¦ò?×°åæx.|yIøÊSîþ£ù©èAF`­Š`I®é7@+åNk­H0Â‰§“-â»ÛWi6çåIKÀ†FmAbpáŒªR¤[«
ëi±Æ‘©“2±âXló0®Ú÷Ó2n0µÌÅrnèQ…©l_£‰7×£hŒí2—¹ÀZe1?yå¤ùz…$R]b¥Œ,…FÆ¾«Sf„©6O´9mÆ\'ÞXêòÙÀT0ºË~˜§_ä=‘f˜=ýíEå´AñÚNrËþÂÆ­´(€©“Ý&GˆèO"ßDÄ…¢ÊÎˆ[YKá\9Íl\™‡"åÔiW(å88þCÀE†»<xÉuÆßr¬hÙ5ß¤šGØ¸?Èž GuËrm/84‹¡ Ô¢É‘§æAþ…â†6=¯TÜTN%–=Nw³<ˆùO9zV}?õ&\A¹oÜ0ù�}¡4 Eï¶
Ã[²0Y~9„Óe`zg®Ã7P«|e«Äøz5—¢=Ë|ÐõJC1°
¡	¿h�\Ø/¸
`Jkþ•ÅEú:ûCJ©ÞULDj¢EÖ‚*ü#ÛNàá¶qLãZ,‚œTŠÎ¥—ù~™t¦æq ö
Hb0’\\Û:EíÜ}âéM©€q8²$|ÑJ­¹ UåäþNókd1¢ô8Áí¨#UŽ¤š$î<zšˆ‚÷[
ÈASÁÜ¬"ñ’®úìŠßSaiãÏþ{ ×›Ö7~˜ôFá–áÁâ´þ–ß	œ‡ƒvï®5“¥7iNðZ!VhÊfb	½nëµÓ@ð_>”T�Û3‚?WN&#y®í¹Xéšðœ™„%wLRãL!Ñ®ü†¨^€F"µ0ƒn{3®¨#5[¤b¾ôÓyàï»:£`úx:­n_
sÚEtFHÆ“¡dÜÒ¡U«zVé{-%ø?z‡
ü:ÐÎ<Ü
�º<DåÅ»Øoi=Ü;»Û(9	õ·stï~›!M©9Žµ¿È¾7ÙxI4ý2O¥ÙÃHÙys<
3z&¦ôÝ¢¼"º�™«¡l
RØµó/™â÷EÛa?’­†lÄk‹Üýë@®ÑÄíóPàp‹,U]ñ˜³Ò¹ü idîcn'´*Ö2+-«$xLlŒŽuôšdFÈÖÈ¨†Ž+¨Â‰Œz!€á¨Ú†Q+)TY	Ýí„À‹G#‹mN]¤È4'
ìt1ÎñMÐéÍ46c—.ˆÍ)­«Ó2¯™Qw5äÇ:Yçð­Çæ{Ã \ö?’	TR9ùA’µ«¾@èŒ(x”$êòÔZ&÷^'hì®ih•ê¸obCc¼A|e5«è“j›VõÔÆHæö²À›Ê½½^6€îÎ Ö8®AI~$ÒŸÜÜ¥’£.`²5ã~Ò‡æ|"iœÉÐž‘­.?
4mÅ·ôŠDç,KL~IÜMNL|ïMñ7*‚ÉËMí34ˆð&6fSôÇ’p¤yÃ÷Ì9‡éÌÕ„ E—]zÚýŸ1g9õêÿyè)#‰AÓ/]vütU*‡]fY�}·7¼ˆ‡~ÀCobë´pÖNƒì÷Ù·Ð^¼tÌs±ª­¾ÎkÎ&8EZ(ƒö”Äb‰×rE
¢NŸ5SûL®›ÌA¥g¥°’^ÔÝYóø¾grg–/ou3¨1Ç¨ÆþÈ™Ù«þmÍüÎ šbWÇÍºDÊP¤˜—Os1t®Ô§b¤Î°¯ÒõæÉjF¿ŠkzŠ[7ÌõsS²‡À%–ùˆk8t2ß‘Ñ†©m$*Ò—=¦¶4{\±¾Ü¶H¿êGPIÚloiH[z1šâæÂD*–ð	hÄ'±¶#
°
rNÇ äK`ŠW€!TE?ä1¿cÞ~ëu¥Ù�Ã_skÄ½â“Jæ=sá"OO(6s{jRÞˆPí'‰°ßéêèü4ÉËšúï¯²Žv-ÛÐlßÀ‰]—IBJdŸ}ÒãK®ÛÑFãâQ™ˆÞYÆul9êÙ›z¾gÓ-=ÙDŽK¿Šr¢h–´T˜¬ZþyÒuïŽºÛtùÕµ%æ…ùöšÛýÞÏ<0|HÃÚKfÊ¥Àë%çMßª‰¸ÜæÌ>Œ· ôÂàÇ±EðƒV‘†þ³ÒÂË@‘(o‚­‚”°”ÿÖbb´ªÌª5Ñøäc�»5OÕùªÌœDœ™¿FŠÒËªË@Ä45„eJ?z¢ð–—ª9P@›´#Íxrm•ß�‚bƒõï& ˆŸÜ´dAÊgj+¤­ú¬džCÔX£Ñ‹JeÉÝVÖJ…ô¹ÖÚ4ä¤®lÒú†…è„¬–¨1°°ÿA;¦z›Ú¯'o.ä]Œ¾¦¢N©;r@ »¶§ÀÝi/Ï|¦GHþRßS2Ó!'ã$•€„Ò#O^=
…SÜ­ÿk�.ì6U[i×uðÁNÍ~P�®åSŒ,éHßÂSâ¶"ho‹6¾Ù-kÐ\E6øªû-¦0aŒ'Ñv³ÊDUéƒ0IÖjØ–*áÙaO>xAhœ"Å<ÀnœñÇá—~Ù	Áé,f×IXãô'ÿÝÆŽ¡¨C£÷àh¦bªmÞ»Ï`'Gqˆ”wE±‡-0ÀMfô/ëÅ­0+2ß<>_1ìíô¦{B£öf‘OV‹¦Í³¶sðp@ÎÇnôF<zøî^Sí4p�±ì8ÖÚQ[¡ße÷HZ1û¿øjtâhí^·ðñˆMóf)€³ÇºÝ+„Èë’eïÀ‘"Ye4W•ˆV6è^ôƒ	ûO¦?¾·ð<ü (Ôüû¼“„µÆw~&h[+[‚)Ë¶î.5¸@¯ô‘Ì–"ñ€=Dr+mKžÅð³”¶#¡¼h:‚}m©*¨{÷ìù*�Tð³ea3³È¼•Såà×†À£­MÝX*&Jâ†–CSÒ2‹ŒïÁÍVÒd0tH #rŸP“…À±#ý8‘lmY€Î¼zù{•°â¨ùñâ.†ÆÚºLçm®ÄþOøíŸ–›ÖÓmUœY¸Ž=ùçL�^ãLÀ¿kðYysì÷}p°ªhêXw%¹IÆ‹ÙÑ-š‘®¨‘q1Þ‰
jK£=4CÄïæ=VF;`èüPÎµâÛ§„]¢,[ftµêy¨q´Ós:9ƒŸõ%xŸîÄoM1à0«È
OœûÄuàMÏÍ£îÃ
æŠklr´�ÅÄ½nko<^ÄÒ‚IÊ±…tV!n¡0	0³¯åÕ=!
çÔ’"¦
ë£ú™?[äo¤E»§øÃLl
˜Oo(ú®i©_èª„®E
 ÎŸšs
EŽyu$Ø¯v'Aáu–c ¹öãr¡Ç¨‘ARÝ(ÖgS*úÄ/Eé	ÿÒTa¿À5l†óÿLÙt1	¦š—cü‡¼yñ<s82ñ»ŒŒ™·ßóÆ©üÔ9Õ1I²`«ŸžÈþ—ç°Ç†,ä¤ÍC×<×¬6_‡y_8Ù¼}w�/ú"öÎæâkoØ¥È!Ãå«5ô_ÏGâaìÖqt{´pÈ´óe`ºwQ`8bßÑhV¯«U¢åQ.Øæ?"2¢üvÅs-¬Œö
Îx±Ê<¨Iö”@ög*4UÎ$&ß­DS]îª9ã‚øF�¢G›^È3EÉÊ¯"mš„tâäMâöRÙÖè_~xAµcÄ5-¥ÿ©
Ï°›“È|aÒÖ	ÝÁ%bÄÚ«§”I‚
E€õsuò,‡O¤<`+§^º’å¸ä0¢qh„4cô‘‘É–X×ÆoÕë9Ÿ«™˜W­Îd~(BŠ)¿Û[pžÃÙ¼[Vv9€Ixs^ÅMîò	‹—s¯<æ9{û	ßú7n“÷åº5ÑÿH8[B;Å½¢È¬=H+Õkè/o¼ €O¥Ö9.gfîzÚç;;u¾žGÀ÷Øq¥µR5¿ßŠóa/Ýì‘{GdnDüýÏÑ2ÒIžê	Ê!}*o%›©„cçÉêŒÏRÐyÐ¢Î¬ºY&ß3Ô®GÃká¬¾æbD{c ]NN«
ã'™²®ª§?¼`p?ˆcï	äK¿gÍ×vz°«Ôt
R°á	ãù1µpM‰É‘8F\8Ñ[à;	¿á—Ýr—xÐÓeìŽIÕ·Úk)E u:Ú02˜oä\)Õ‡¤atâg¤#ƒHJÕÆê¿p¶˜:lÆZ¬Is)I(°â«~ŸSS¾RyÅ5k$ÿŽÈËÊŠ¤Å.¸
±öFKe¥u
^|ÒPßííužæŠÃI}OB98¦òX/Òë¡`¾SSª’ÒÑ6‰µÇøm@j®b‘yÉX|šØÒ`ù%\jµI‘[¦ñN
w%ùÔ2BS•c•‡pE=Q‚³¤e‹W[¤·\Í×¨c/AK[5k¬o…—	ôð@ýò·D ^ÒtK@\íäkÛHG=°X¥GïŠ!y±s·Îm’\pœ¯¶ð8yW?ûØÌMOSC]ˆõ#;Óœþš!ãÛÏeÜŸ±)fõ}v…¯Ö[ç˜Ó?š'áÒ2Kô–±' X3„ÇÄEŸ‹q¦‚ ˆA„ÈÈ'gZ½äCJ¦Ç¸ÞÄ¦e³a1?ÝMÀÄÌUY1Ò¤¬>X‚Žkh2¼—Õöº\ö«'3êXm)sÐ$~¶TœI…wÊ–#åÃâÀÑ˜&žhzCçû¼.“öæhþ&ŠG�˜--IF›"“ß$Ÿ¨
\…KþÅŒÁó–¡te{$|¹IY¯‰bŸ)2¢ðÁ8�õÕDRf’Ãµ‡2¹cÇ«üÚ7Ãù?!'"8<c#èù4å\•Cð|¯wP4YJÕ¥í..Y­ˆ‡~0È\ÔŽÜOa‰8¨ðÉÅe„Â…l=Ä½À¿*?:²æu™ÜÅ™)Ý`ùOêÑl~Œ§xÍ1&�Ãß€Ênßö
½ƒÔ ô=|zÌO¢ÒýñX{„T0!ÿä´svÏËa‰ÌŽq™€@#õ¼­Š¶ú,ó¾
±?*±RÑòŠNF
çùbý_¡ú³É"ÿ|-Æ¢½Ì7ùÔAà½ªŒ)­äR`w!žq+U›Ýg
E—†¸›0u˜¤óDç.t°Àá´=·VÂFêX�JæT:òài2,f¤+	ˆôWSôøÇºÀa¾ï“2ó2bô¹Dá´ µ{­{èÖv’ÑÒT¡.§IÑ=gøî‚µV•qÚ|tKÈþ’{S±×2œ%NŒïï?®¼ÞÉé¸úMWm®K.}ÃMÈdÏ
ñ÷ª©ˆ&ýÊøñCÉå4£
ç£ÍîÐsºY&‹Qó?/ÐC!çÙPòb$,¶¹…¯ª=gt+:+Î9³».¶¾ß<“VËÒ5Ú®	ŒÅ¾—XŠœóä<.Ãaµ¨©Æ¢*ÎFÄ§,ŸôSÎ¥Ûvk±„¾ž‘D-qQoHXÑ®k;öê©EÀÎix'+Ò×i„ñÔgTMµ`ÇäCVlF
Ä¨Qó?ðkØ
Ú2ÆV‚}»2xí;içÁÝùÈÞpyçUñ£	¨…÷Jœ- àøº¯êÿo†CVŠÖèf4÷WØ–Y­Šj-¤-cC€-:Â`æ}=Õiß¯•óg2KÒÌºõV1Í€ì#ƒ£œä1_ZOOäÌ—ÛÇoªÒá†È<V¬:`cŒë1¥ƒÞÝâhØ+´�øö¢‹­@¢hšPöÍm$š=ÓäÔH„f†6°j1JQ‡dóª^Qœs¦Ê¾¶F©•†ž¤Þ*fØÚîìúX\e*Ð‘yI™ÖnIíštq¥)ìn=k¨h´ÙÎ·Jv>?Á$@…ƒ°wÂ>&˜•?¿_£Â!ÐÆ?¼B²ËÉ©‹ˆÁ·=n)ébLqÙñ‚ã¥5<Ðukïóþ8Ø’8ÉOÑ‚•YÍUú†û÷ŽÓ¹ã@nµe¨ dŠ€¢fçêxHŠ’‚5¶Ì J­×•û!P¯"ßN¥o&>@ëT'¤e×zZÁ6îF°®…kImèÂ òiÚØŒÃ¯0�[ZnÅ¯ÿWò¢Üt¦ÅÏi­ýë^ÁY’bâ©‡³<€1lÙ]à²wbéáÓ`Y×Û6PÞGƒj7Ï¥äÛ=Lgá SËãÊ�È¶/›	¶ƒ+£‡–fÉÒßÉ©Qµç¶æ’á=¥¡<«6–ÇÀFJ@% 3Ç;¡	ª¬o¶~ ÜJÍäÃWªg¡zdIïôt×u2Aã‘|wAdH1G‡5*Ü÷¥‡´Þ~¹µÓ+$2¼šÄ­ÞÄäÿB†ÿå¢kB¾ÍˆF‰œ,_gÌy|A½ˆÛVê‚”%Óqï{º3óM&+ÌC4ù±Šƒóø+¾g¶ÛtwœËíÖ×$øþûe4ËÇ¡÷“'Â7'Y½8<
Óù¹1£Y/ïk!I`ô›¶¦[Ù¥Åh—¦¥Foà®]Ìç,i©pŠQº"(Ï‘Ã¬ßæe2·6
Ò{q€'¥?Å2ý42þÿ”#ð	+3ó
ìŸµÀ²g©#»…/tSd+¥2v61˜Ããå_UNg­ûDá8Výn+Û‰^Ý+íåýŠ–úkn„•Œ÷H'¾úuŸÆÙ-Šªm¦@PÇÎ…CªM´Â…Í&8Ãª×›0@¸SÐ†F
(Ü3¨.O*®è8“Ä<È–¢.vâ™¼8RÂáFIéZO¹<ù™+@Ó^ò/ñ¹.¦d×ƒVHB`39†_�&{¦ÏÑ„ÑãÍõGu¡Î+þ
›ò¥²'ßü`à…¢y`%:\3Œƒœh·jû}e\Fã®¸AÖï%2&N=·qüUêñnÎŒžP!@ƒ…›¥„ç(|xq{¸ý€Á¾]
ƒç¹æ¨Žïè«vÑ-�ñ¦Wm£×Œã†‘T³Þ,ûö(`�~æëÍOÇG	¸‚—HçñÃs)Ÿ(ýV.œŸe_ÙãØ;Ó?ÓßöÛ{ÌIæÂ%úb™•ïnHš‚SŽ‡£!åŸ:Æöuë´Æ¨Ýü”K¯ÇàµæÌ˜‡ž7ÚÅRõòð^Ê¼ë~“Jðnßø'ÿÿ[ÏœíÛ¿ý¸–h¦×ÒzE°¾grÞÑ¦Ùoý¸çýR$Ló+Ñs2m}µ&~54©0LU˜z)Ô0[BäËý…62
)#ñkŠný>ƒ_žBÐÆÈX?žÜìuhr¢¥qÇyÉ¦/”»x
þäQá¨w«­ÚOÅìv¯Ò%]Ø"¦½r7¸§£×¢-Ð G8˜L¡Ì)šz—zšt™ÓJ£øœÏ¦Üe,Žÿt+õŽ�
È	¾	Mz?ô£…CÌ÷îï<…4âAÜü
>ûÏ¬M;[ÍÖ¡?“ Á’´¬Z÷d†zcI
«»—Ð¿çât*µpÍí\ÿN$A©yƒ¦Šú•éÔ²äz3¹©Ð„ÑË-.‹§F£À˜0è¬3'ÒX¬rÚËy*ä‹]BZïé¬½y¹l©}d£¨6g#reÚ°Ànv ƒ˜EŸDhª66È'šŠUhÕ«B%	 Û“|Çf»KŸ]!U.®¡ªCî¨Ñ)2ÓÃëÚ¡Âˆ’Ú2û;8²ê}‡ª{|sŸC5n
¾×ÖA/íð©œ`ç*5†C/³´ø¼!\fÇ3ïgƒ÷QáKòo¹p}ÇãŽgû™k¿Í¼¥£(ä²˜™¤ÒFØ×~ïÑDMT¢õÇçZ#Õ»Kí¡Âcu¾“Úq3ñòüû rˆdOaà¨pè?Ü¿¬|u„Ôž®ð
?_A³gênWQ´õËCFL$ŠÒ;µns€&¢›‹NñÒ%(<qÜniT6ÝìA1ÌIK6šcò~y°ÆoïçLëÍ+bçiÜ,Í-¯0³‹‰œ‡ÉÐnæî¡Øöú€RZñÌ‘kÒÃÕ¦3óÇáÔâ¾f7ÃF×ú+§c¡“øêŒVn©—’'%††Žp¿§§R³eaÏ@-
Að;»QÖµN£±-o¾å|al$ì- äÉ†ø;•è{\¯1ÙÂ$±ÅZÒÜoôË+Ù12ÊšÚe˜.PŽeÎ(@)2Ùl+6ÖKùY¦{M¿à‰×8w,Œ½¥‡P±FQú*ûÂûÊù±‹!µlŽRãË$müƒ›ß€åœWßÛX`”âÆ‡yK¶ÃE	ÊØäŽZ»#DÕ:×>¤bÕ£¦dh½–3‘�”�sâo†EqdÇóù8,Ñ® ê—m!$ïÐ°Íää!†íÉmîX9SoURVTã!<{uªÜ¡Ì'Ñ_l‘g mzÚâ÷Û¼ø`ûÞ–ŸŸ¸„â¯ŠBÎå’f¿ù8ß_mªè«DÀ­¤ü,ÂÆÆ¦»€œ
º%Õ$+Ý>ÿåGHÕÌÕ{¥7‹Ç/ÄÝG$fƒÂs¾:á¡œŠTPV¤÷°¾»Â>æŠó—6ëÌ† ƒª–È•µÇ‰~y Ä'³’Âœ´ûdÕQ©ªþ?eo's‘�8-ÇÚûþn,Ø›ŽëLÎ@`
v�…{fJËàDnŒë¼³;¿ŠÃÕÑ¥¤7¿—U”*ð5#ì‚Ùûtø‚'¬?qaÎS1 vN@ÀyZKÉ°óñ«òTaôµÕÀ²™±Ñù;Ëwûôj3Q!ƒå .5‰t>¿¯s3h3ì÷ÿÄ¨¨¡Æh
ÜSIÓðÖm}°Jß$[¨b^#.P7æ #7Ð§QrTå‹:&ý_h5¥·eeøY—þÉz›Di%ÓÒ	¿»[æ¬•|Töÿâém`pMg‘%ŠÌKË ×àÿ©ûŠ®Á¼‰}ò}pq:bÉ#|WØ@[íå¾B9ÂÔ˜@ÒB� Ì˜xE¼Èp'™(±{/Iåß5O+ÜŒ£00#VÓ¯¯ðÌ‚ùÓm4¶ð<ëê¢Œ…\Ë÷VºtË-ZXµ¬»ÎÞ°ìp~4;oŠYYDš½|Òû&M\O¿‰5¶ ‰GÝŽÖÆ;ª<ÇŸ&¢.,Jõ}ÊE€á.†éj…Ô2ØîsfÖÚû¬º»]:ÓL/†GNHoújQë‘paÉ
àv)èÏb»¦ên’¾ŽÁïÏSóZE°
ûGk`ØPµÿØ|Ž¹‚bQZc3ÝdózVx°ixaúŠÓ½~j¶"¥fa×ÉW‘nŒo};\MŒ÷t¶—×˜Ÿ.°Ð®i$¾Êoð´ä¸	7ÙKæ´i¸$ÞŸ™JVÁ<\UvZªÞIxW'C¦Ñ5’Pž!ÔÔO
Äãö¶”°vpŠs¸!Ë‡¶“cÙÈ	ÌplU@v÷¢—) î§ë„äŸKE9€á¬—KãþA l„c¶JDè#Y&Ê–ïù©çÂ~%­é¯7H‡Gf$?¯ŸÀv˜QÔŠM:cˆßH¡Ï‚.#ù*™¸ˆøþ>iYºuêÔ¤¾—tðÖZtM6`Õà™l,G-È'˜Q÷ËÉßËÙÏÉ??²ýíîa@Œ)–…”òÖ	¯ÿáü ÇD.•fpñPæÞÅØîVs(n¶tÕà˜¿yjPt6ÍeÖúkœ{Ê”ÉzW„kÑ ªíê¿—ÛhCÚúB•¾%%p0d¥Ëbþ7n\ü5¾¼o[Â®!ÁF9	Ëõgê(%…ŽRŽÊØê(++ÛRí9ö øŽK5ïâ¯–É¦ÍS?¨ÍßÁ¨¥3¨…&Ç¤¬Êléc‚’ui¯|^:„Vœf2OÝÜÎBŠ‚Òî—Ü#5½¹-yË¶úu f“{j3>"íòÁ~DÉ>ÿ^ˆõ‡	;Ä'ÒyÚ¦=ÙÆÒJá˜Ë×¤~z‚¨5ö÷)8™-¹*ö8âž$“u–^šÐÒ.ÆªFKq¼ÙòŠ
;°Û%Ï%Jf4öÄ:~r3_(âÑtÚ…wÄáÁv
`³‡
$ÔçÁÛj2Â½ŠÐŽ1è‘Ðq»Pn¾áX÷w>¼ž{ï©žÁÿËÑ`ÁùV.r¡·D‰öO´¦PDC¦oR,Š	ÆÒ²€„sîH&Tüi:6(ü<›á8cÓÜ•ÎæLøòâüžµ«µTlfëx/ÀQëÍ=•ä¿;¶‹›²Xt™2%½jsñeƒ?
:`t—p®p°š`e£µO'ŽÐ­;âµùÌMá íÉzîÇÚf_’Ú)€½ëo
dsÍ‚'×ÙTÁúwžùÚçÀTÕ¼Ë•|Þ×�ä~ô³1ÍP;ãoT†PÌ@M°“›Ø^ì•â»ÎäÝhk$Ï^g,¤é>ŽÎª"#ë‹$Þ&9VÔLTfj)ENëz—ÿz@ØÁÏ¶/|&QI)‚ëãè¸æf±Y-)kª
×o9í~š+
B÷ÉK„XTÕ\å”ÐF^õ,íÞœ—[1?ÁÜùVn¨¨…FFRA)KÍ/mŒ˜»ñÎ¤nh85
ß%ƒSÝds�Æ… —ÁK#P$YJÔ2=«˜EÊà…Ì£ÍvZP=Ÿ 
|‘²\ƒ¸.Gô.S-¸R9»ÑZ¶õ×r‰V’îT&ßä…`?ILéDNÐ¬ -²
kQê°.?3Y&è«äÐ~w]ôÏªH¤ŠkH²ù
;ÖÒü>ø!¦Sóc“òg.G^QÓSxg!
Ý¹9"ãÔÊ·{ýeßlÒDû¼sþÕŽ{v'µüÜxŽ²³«_½EÉ9L*ÐãFÐ&î°ƒÝeæDq®ÞŽ¾dVâ½üÆÄkœL	xn0òôHºÃâ:”ÿh wóÆŽ"íœ’JgÇ:vV‡Q…!¡ÛâòUˆ&Y‹õßÐe?ƒ>æœÝýbU›÷Õöší,› ”?pŽ0A!}¦G¥.#%µv—^°·ùÎrËEÅà›}ÓwƒÓ÷cÔ'PÍØÝ™,µÎæ—–/Ñ?àÍHpí~C&QœÜŠtØ=ÃåC?oÌôºî0Q ªLÁÜ›{($§
"…à¹qÿxvé·+jñÐIò(¤š[ª=%Ÿ±héDl4Ï!¯�®bœÈ0’^/;l¬¨¶4‘õ›í^5Ív(ws“–›ýÓ¸ž» r.,Óvm`Ø¢òó-†2y²…Ô_F`¢cÇÖÐeö_P]ü
¤àÈõÜâ›i!}Ùñ™*½ð“%'Ø°/å”i«¢{2eôŒQ­P«À/v¶Â[ù±Àb—K5ñ�T�ÚÓÝúz‚žžõÊñüÕÊ9ú'§lo=V§®å­Ú[ÒÑ©R5…Š¬ë÷I/î¦;5à€ã¤e$yÄXÐ0¯t?_ÇY/ÔqÂá“ÛeCulmÑ[2!‘…¬
„MÔ‚¯)„R°ä}4Àž·HÈ,cyôÂR”*…£á~#è@�Z_§Ô|\2qý52_µ]%}Ç°+£ŒI
y8*ÏÃTà°¤ëƒêÈì®&€í6éŸˆÌÑ½€|î¨Š„A¤aÃP@ØÇ.
Ð»ñû^–Â½þ¦ö˜ÇÀOleãÜGJV›„§6±~ðxö ‚	ð½Èà¨û^1 Å‰hµ®%»qLSí|[¤ì#‚îws¦Cp¹ ­ˆ
æ8Ö”yÍ®ÄÎ¡ ö¾u¶yÅuXüæ"á+Vü”ÄµÓM	º©­çÇÍôÞó¿,Cì,]xç±——åt/çer]ëu’å5ù=¶Èa›öM™Çhß¤â÷MÚr[°énIfÝq$v³b›ò•¬†v2ó“¼šnm)ÜCÏLžcl>k;sô5ÐD4ºw8­k^)Ð~3Ù2ÌÙÙd<ËÍÒ.ï3“¡¨ú“L˜˜Ý¤WŽº|È6ªˆÜVé&’£ðãŸ‡—¬ÏOn«C÷H÷
ýk
š}G˜Ù‘
Ùˆb}OeÉí’Xy74Âý¡ÅÃ%Î+ÅŸÆ;!Ì”HˆÌXO»åsƒo¿sªò´úö³¹O>pt¯ôˆë¡ú¾ãÀêoU ”7•RPøÉ½«G±¦¨ðm5O¦\é"+NuûßOaHy‚Ý)Äð3Yd­á˜g´ð¤z¥Lh‡‚
3ÕØéÕŸ1­/}†§?QgkŠõŠ:Ù1QÎô0”‰÷1©ˆv	I(mÜø(!]á
•½ú/?ØŸ3:Þ…\5y„()|š1øqë´Ãï7ùW„ë'‘j³ÓÈ- g�jjatºxènM ›«ÁH¼Ïuá,üëµZ^gH=þæÖÓØ?—C·àÃf¢TQ¶~ÚŒŸ;=ñ›4H¹�®Õ°åDû×±}ÊÝýì‰¦'üŽô¢åç`‚v˜)Û ×xú€è&™Ç¤ºß6ûúZQŒãfl¿g§Óï±|93Eªæu÷ÚÆr± (X9N¢*è®¥�8ˆ¼ò_Â)‹\Óº[ú(°@| Y,™È×h0ÒijŒý@˜µtCÊ\ë;
-’¤¨C+ Å(™WªlÝ)ëJ/žu	º}J&aëüÇ£™‰«1ìB¾~ª;ßv„BpjD3ÃåC¢ÁKQø!ÊpžÎanÐÁ‘6”à”øìšðºˆæŒÚ¼N~-4¬²âª8Ã¿U2™~cÀ:XúÞ5€g¿�"‡86¦Îý!o÷P‰`¸Ð­·‡m…¤Rq8Ø«ìqcŒÇÝçò!@2ù£„ËÆV€†ŸnÞ~¥Ú¼i5'^ÄFRU¿éyÕ«înh±œu¸o	’¢hù¿ædR”Ð¸ÉÁÇ«·îh6ÉO¡‘ÉúW?Ã™ïO·8¤"ÇÁîgü#ÞîUnQñofpšÂK•Äs
Îf)©ñh‰Ö(ˆ=Úñ³§	»&nJuúd‡­Ìýð9¢‡ÀDûz=êjrËÀ·á;?.ÊÝ Î€œø‰«Ý°?fêÒÈ|à)-„yŽÏlç®B½X¶Î¼ŽAðH¾}Å¦¸>¥„Ð½¶	E!¦¼ Í=|¬†h_‘˜:ó×’EÖ#¸Å@³"ã#[íœ×(ølR¢=º¿Ít¥/™KáªqeìÈ°ñí`PnºK5cÈŽù\\ÑŸ“3#4Õ%ë($·Û}c^O+Ê©p¢•R;+fÒ“C®Áò$”éQ~G©©ÓR¿ß%‹e
dafœ•¢DÇä=§Z¨$ßëÀ éÏl§…Þö·äæQXe™eóoÂR-d@d"žLÁŸöi¦@I]À<L*"R•]
ÕiÕ";?Aç{b
ÃwïÄ¦qYTL§GìÑ±@>¸,Ñ
53#|ƒì)©› Ÿýõ ÑI’kö€D	d²Ñöb5qîÄ¢j“a8M8CìM“üŒA–Paÿ÷Ò|Ôký€þM¼è'Xbeµ®‹äøàé9Ì+9ñš6t{áÌ…’˜šsPII1ÛiÂ+IäI¼
f$T4øç"5*™>Ãº3KÕe¤€ÅÞ½ÌmªÈN#—a¬1S¶+Á´¡ª_r:²=<K¨%ÀæQ¶(¿1â}œ®©X9KÙÀ ËÐt£©^q–kiÒk÷Üº¨–d6åöæ`m‚£’VÚ?9t³m
\Gä–°¢òraÊæÐ‘!R2â€Â’¼
%9¹NÇÎA~|f¨÷ÿñ\wa*ÍMïúöÀY^öIKa[0®·#ÝÅÚlv4uRg½–Qh¸Š¿“½”àß¿_ær6‰¢Voø™—7Bäp!\
J›ÉEóßú+>¡!Êä"ä�ç/QÏå¯fuÑï•Î›qô¦Pçp‹zJœw²¹eAÎ§@wn>ãÝÙ²°…�°ÎF±Kß³ÜX,‡ÌR}U‘FX÷‰¶´Ë«;b\æu×þoÕäCGQËº–Å±Fi(ê§žú0ûïfÿ–yR=(Z)®3"óI9o,®é¹¿`	)IA<C¥Këé
"»¢^+© Tî›Þ€eÜ«v“ù‰ðÑYX´('hÂ„2£#t0P’g”š”qÁÛnXf(Tø‘Øèn)H¼zQ‡ÚN
Ø¢jnØóYëé5ÔXM—IFÖˆ ðéFö½+}V%¡º`é>¢*%	ý‹-ìšv˜î-s²7{o-½RîË$¹ÂîÐN+î|o.Ec`³:>íSî¡×1u»$
LBu“mk¼ÖáôðèèîÅnV:FØ‘7‹.~½¿dxyÄû%mœÁäÝjÔL£âêc¼E¸AŒ~N¿¬Û¦AY¡3MˆõÒÁ·@#Fph´ë×2âk¶‡`£Ù^åýá	['‚ ódž³šEÚäfš�ÞñŸêxß:ÅäûPmü5u/ÂqWS,l¯ÍŽ®Ä,Up@eó¦ÂcÅíßãx<Í•A/—JŠº‰ÜÌÀø(Ãµ€u
{ÕþÌÿ_IàciÅ,¬ÖLK¬©$)B%s²P	‘Yí.Ã
ÇÈ]Pâ©c¹?<s|¨BÏÃxA#lˆ
º1S[þ«tÓ)íK•ÀÁÃa”Þ™ì~œLó;f|Ô"ÅžÈ8WJîÄ¥—¹"@£~á9kÓþë©™Å/c„EŽßŠ§¾å~ríUÕv…Ñ ®ÏB³r^Î(–ýô
Xî?x0ö!s-ß<S¾[é}ú§{2€}Y”IpL”ô4µ<ËÜK'ªÏ–ºö'Xø¡J4;eT§•X·>H
½N=
]îÏAl<T¯­ùÇ”iÑø7%áŽžîIó§Qï‘^&6m/¯§eØy`[3‡“°âðß>Iñ6ÆáÑÁùÂT‡üSÙ¾®	²e¦àýÎø§á ee"#########;
        assert_eq!(content.as_str(), content2);
    }
}
