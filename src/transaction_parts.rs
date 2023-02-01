use crate::traits::*;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Vin {
    pub txid: Vec<u8>,
    pub vout: [u8;4],
    pub locking_script_length: Vec<u8>,
    pub locking_script: Vec<u8>,
    pub sig_script: SigScript,
    pub redeem_script: Vec<StackItem>,
    pub sequence: [u8;4],
    pub value: [u8;8],
}

    impl Vin{
        pub fn new(txid: &str, vout: u32, previous_locking_script: &str, redeem_script_vector: Vec<&str>, sequence: u32, value: u64) -> Result<Self, String>{
            let mut redeem_script:  Vec<StackItem> = Vec::new();
            for word in redeem_script_vector{
                redeem_script.push(match script(word){
                    Some(n) => StackItem::OP(n.to_bytes()?.try_into().unwrap()),
                    None => StackItem::Data(word.to_bytes()?)
                });
            }

            Ok(Vin{
                txid: txid.to_bytes()?.reverse(),
                vout: vout.to_le_bytes(),
                locking_script_length: varint(previous_locking_script.len()/2),
                locking_script: previous_locking_script.to_bytes()?,
                redeem_script,
                sig_script: SigScript::Byte(0),
                sequence: sequence.to_le_bytes(),
                value: value.to_le_bytes(),
            })
        }

        pub fn sign(&mut self, signature: &str, sighash_type: u8){
            self.sig_script = SigScript::Legacy(SigScriptLegacy::new(signature, self.redeem_script.concat_legacy(), sighash_type).unwrap());
        }
        
        pub fn concat_legacy(self) -> Vec<u8> {
            self.txid.into_iter()
            .chain(self.vout.into_iter())
            .chain(self.locking_script_length.into_iter())
            .chain(self.locking_script.into_iter())
            .chain(self.sequence.into_iter())
            .collect()
        }

        pub fn concat_legacy_empty(self) -> Vec<u8> {
            self.txid.into_iter()
            .chain(self.vout.into_iter())
            .chain([0].into_iter())
            .chain(self.sequence.into_iter())
            .collect()
        }

        pub fn concat(self) -> Vec<u8> {
            self.txid.into_iter()
            .chain(self.vout.into_iter())
            .chain(self.sig_script.concat().into_iter())
            .chain(self.sequence.into_iter())
            .collect()
        }
    }

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SigScript {
    Legacy(SigScriptLegacy),
    Byte(u8),
}

    impl SigScript{
        pub fn concat(&self) -> Vec<u8> {
            match self{
                SigScript::Byte(b) => vec![*b],
                SigScript::Legacy(data) => data.clone().concatenate(),
            }
        }
    }

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SigScriptLegacy{
    pub script_sig_length: Vec<u8>,
    pub signature: Signature,
    pub redeem_script_length: Vec<u8>,
    pub redeem_script: Vec<u8>,
}

    impl SigScriptLegacy{
        pub fn new(signature: &str, redeem_script: Vec<u8>, sighash_type: u8) -> Result<Self, String>{
            let signature = Signature::new(signature, sighash_type).unwrap();
            let redeem_script_length = match redeem_script.len(){
                0 => vec![],
                len => varint(len)
            };
            let script_sig_length = varint(signature.clone().concat().len() + redeem_script_length.len() + redeem_script.len());
            Ok(SigScriptLegacy{
                script_sig_length,
                signature,
                redeem_script_length,
                redeem_script,
            })
        }
        pub fn concatenate(self) -> Vec<u8> {
            self.script_sig_length.into_iter()
            .chain(self.signature.concat().into_iter())
            .chain(self.redeem_script_length.into_iter())
            .chain(self.redeem_script.into_iter())
            .collect()
        }
    }

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Vout {
    pub value: [u8;8],
    pub locking_script_length: Vec<u8>,
    pub locking_script: Vec<u8>,
}

    impl Vout{
        pub fn new(amount_satoshis: u64, locking_script: &str) -> Result<Self, String>{
            Ok(Vout{
                value: amount_satoshis.to_le_bytes(),
                locking_script_length: varint(locking_script.len()/2),
                locking_script: locking_script.to_bytes()?,
            })
        }

        pub fn concat(self) -> Vec<u8> {
            self.value.into_iter()
            .chain(self.locking_script_length.into_iter())
            .chain(self.locking_script.into_iter())
            .collect()
        }
    }

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub size: Vec<u8>,
    pub signature: Vec<u8>,
    pub sighash_type: Vec<u8>,
}

    impl Signature{
        pub fn new(signature_hex: &str, sighash_type: u8) -> Result<Self, String>{
            if signature_hex == "00"{
                return Ok(Signature{
                    signature: vec![0],
                    sighash_type: vec![],
                    size:  vec![],
                })
            }
            Ok(Signature{
                
                signature: signature_hex.to_bytes()?,
                sighash_type: vec![sighash_type],
                size: varint(signature_hex.len()/2 + 1),
            })
        }
        pub fn concat(self) -> Vec<u8> {
            self.size.into_iter()
            .chain(self.signature.into_iter())
            .chain(self.sighash_type.into_iter())
            .collect()
        }
    }

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum StackItem {
    Data(Vec<u8>),
    OP([u8;1]),
}

    impl StackItem{
        pub fn concat(self) -> Vec<u8> {
            match self {
                StackItem::OP(o) => if o[0] == 0{
                        vec![o[0]]
                    }else{
                        vec![1,o[0]]
                    },
                StackItem::Data(d) => {
                    let mut varint = varint(d.len());
                    let mut data = d.iter().cloned().collect::<Vec<_>>();
                    varint.append(&mut data);
                    return varint;
                }
            }
        }
        pub fn concat_legacy(self) -> Vec<u8> {
            match self {
                StackItem::OP(o) => vec![o[0]],
                StackItem::Data(d) => {
                    return d.iter().cloned().collect::<Vec<_>>();
                }
            }
        }
    }

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Witness {
    pub nitems: Vec<u8>,
    pub signatures: Vec<Signature>,
    pub code_separator: Vec<u8>,
    pub redeem_script: Vec<StackItem>,
}

    impl Witness{
        pub fn new(signatures_list: Vec<&str>, redeem_script: Vec<StackItem>, sighash_type: u8) -> Result<Self, String>{
            let mut signatures:  Vec<Signature> = Vec::new();
            for sig in signatures_list{
                signatures.push(Signature::new(sig, sighash_type)?);
            }
            Ok(Witness{
                nitems: varint(signatures.len() + redeem_script.len()),
                signatures,
                code_separator: vec![],
                redeem_script,
            })
        }
        pub fn concat(self) -> Vec<u8> {
            self.nitems.into_iter()
            .chain(self.signatures.concat().into_iter())
            .chain(self.code_separator.into_iter())
            .chain(self.redeem_script.concat().into_iter())
            .collect()
        }
    }

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RawTransaction {
    pub version: [u8;4],
    pub vin_count: Vec<u8>,
    pub vins: Vec<Vin>,
    pub vout_count: Vec<u8>,
    pub vouts: Vec<Vout>,
    pub locktime: [u8;4],
}

    impl RawTransaction{
        pub fn new(version: u32, vins: Vec<Vin>, vouts: Vec<Vout>, locktime: u32,) -> Self{
            RawTransaction{
                version: version.to_le_bytes(),
                vin_count: varint(vins.len()),
                vins,
                vout_count: varint(vouts.len()),
                vouts,
                locktime: locktime.to_le_bytes(),
            }
        }
        pub fn concat_legacy(self, index: usize, sighash_type: u8) -> Vec<u8> {
            match sighash_type{
                1 => {
                    self.version.into_iter()
                    .chain(self.vin_count.into_iter())
                    .chain(self.vins.concat_legacy(index).into_iter())
                    .chain(self.vout_count.into_iter())
                    .chain(self.vouts.concat().into_iter())
                    .chain(self.locktime.into_iter())
                    .chain(vec![sighash_type,0,0,0].into_iter())
                    .collect()
                }
                3 => {
                    self.version.into_iter()
                    .chain(self.vin_count.into_iter())
                    .chain(self.vins.concat_legacy(index).into_iter())
                    .chain(self.vout_count.into_iter())
                    .chain(self.vouts.concat_empty(index).into_iter())
                    .chain(self.locktime.into_iter())
                    .chain(vec![sighash_type,0,0,0].into_iter())
                    .collect()
                }
                _ => vec![0]
            }
        }
    }

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedSegwitTransaction {
    pub version: [u8;4],
    pub hashPrevouts: [u8;32],
    pub hashSequence: [u8;32],
    pub txid: [u8;32],
    pub vout: [u8;4],
    pub script_code_length: Vec<u8>,
    pub script_code: Vec<u8>,
    pub value: [u8;8],
    pub sequence: [u8;4],
    pub hashOutputs: [u8;32],
    pub locktime: [u8;4],
    pub sighash_type: [u8;4]
}

    impl UnsignedSegwitTransaction{
        pub fn new(tx: RawTransaction, index: usize, sighash_type: u8) -> Self{
            let mut hashPrevouts: Vec<u8> = Vec::new();
            let mut hashSequence: Vec<u8> = Vec::new();
            for vin in tx.vins.clone(){
                hashPrevouts.extend(vin.txid);
                hashPrevouts.extend(vin.vout);
                hashSequence.extend(vin.sequence);
            }
            
            let hashPrevouts = hashPrevouts.sha256d();
            let hashSequence = hashSequence.sha256d();

            let mut hashOutputs: Vec<u8> = Vec::new();
            for vout in tx.vouts{
                hashOutputs.extend(vout.concat());
            }
            let hashOutputs = hashOutputs.sha256d();

            UnsignedSegwitTransaction{
                version: tx.version,
                hashPrevouts: hashPrevouts.try_into().unwrap(),
                hashSequence: hashSequence.try_into().unwrap(),
                txid: tx.vins[index].clone().txid.try_into().unwrap(),
                vout: tx.vins[index].vout,
                script_code_length: tx.vins[index].clone().locking_script_length,
                script_code: tx.vins[index].clone().locking_script,
                value: tx.vins[index].value,
                sequence: tx.vins[index].sequence,
                hashOutputs: hashOutputs.try_into().unwrap(),
                locktime: tx.locktime,
                sighash_type: [sighash_type,0,0,0],
            }
        }

        pub fn change_vin(&mut self, vin: Vin, sighash_type: u8){
            self.txid = vin.txid.try_into().unwrap();
            self.vout = vin.vout;
            self.script_code_length = vin.locking_script_length;
            self.script_code = vin.locking_script;
            self.value = vin.value;
            self.sequence = vin.sequence;
            self.sighash_type = [sighash_type,0,0,0];
        }

        pub fn change_vin_p2wsh(&mut self, vin: Vin, sighash_type: u8, remove_up_to: usize){
            self.txid = vin.txid.try_into().unwrap();
            self.vout = vin.vout;
            self.script_code = vin.redeem_script.concat_legacy();
            self.script_code.drain(0..remove_up_to);
            self.script_code_length = varint(self.script_code.len());
            self.value = vin.value;
            self.sequence = vin.sequence;
            self.sighash_type = [sighash_type,0,0,0];
        }

        pub fn concat(self) -> Vec<u8> {
            match self.sighash_type[0]{
                1 => {
                    self.version.into_iter()
                    .chain(self.hashPrevouts.into_iter())
                    .chain(self.hashSequence.into_iter())
                    .chain(self.txid.into_iter())
                    .chain(self.vout.into_iter())
                    .chain(self.script_code_length.into_iter())
                    .chain(self.script_code.into_iter()) //FIX
                    .chain(self.value.into_iter())
                    .chain(self.sequence.into_iter())
                    .chain(self.hashOutputs.into_iter())
                    .chain(self.locktime.into_iter())
                    .chain(self.sighash_type.into_iter())
                    .collect()
                }
                3 => {
                    self.version.into_iter()
                    .chain(self.hashPrevouts.into_iter())
                    .chain(vec![0u8;32].into_iter())
                    .chain(self.txid.into_iter())
                    .chain(self.vout.into_iter())
                    .chain(self.script_code_length.into_iter())
                    .chain(self.script_code.into_iter()) //FIX
                    .chain(self.value.into_iter())
                    .chain(self.sequence.into_iter())
                    .chain(vec![0u8;32].into_iter())
                    .chain(self.locktime.into_iter())
                    .chain(self.sighash_type.into_iter())
                    .collect()
                }
                _ => vec![0]
            }
        }
    }

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignedTransaction {
    pub version: [u8;4],
    pub marker: [u8;2],
    pub vin_count: Vec<u8>,
    pub vins: Vec<Vin>,
    pub vout_count: Vec<u8>,
    pub vouts: Vec<Vout>,
    pub witnesses: Vec<Option<Witness>>,
    pub locktime: [u8;4],
    pub has_segwit_input: bool,
}

    impl SignedTransaction{
        pub fn new(raw_transction: RawTransaction, witnesses: Vec<Option<Witness>>, has_segwit_input: bool) -> Self{
            SignedTransaction{
                version: raw_transction.version,
                marker: [0,1],
                vin_count: raw_transction.vin_count,
                vins: raw_transction.vins,
                vout_count: raw_transction.vout_count,
                vouts: raw_transction.vouts,
                witnesses,
                locktime: raw_transction.locktime,
                has_segwit_input,
            }
        }

        pub fn concat(self) -> Vec<u8> {
            if self.has_segwit_input{
                self.version.into_iter()
                .chain(self.marker.into_iter())
                .chain(self.vin_count.into_iter())
                .chain(self.vins.concat().into_iter())
                .chain(self.vout_count.into_iter())
                .chain(self.vouts.concat().into_iter())
                .chain(self.witnesses.concat().into_iter())
                .chain(self.locktime.into_iter())
                .collect()
            }else{
                self.concat_legacy()
            }
        }

        pub fn concat_legacy(self) -> Vec<u8> {
            self.version.into_iter()
            .chain(self.vin_count.into_iter())
            .chain(self.vins.concat().into_iter())
            .chain(self.vout_count.into_iter())
            .chain(self.vouts.concat().into_iter())
            .chain(self.locktime.into_iter())
            .collect()
        }
    }

//allow vectors of Vin, VinSigned, and Vout to be concatenated to a single byte vector
pub trait Concat{
    fn concat(&self) -> Vec<u8>;
}
pub trait ConcatLegacy{
    fn concat(&self) -> Vec<u8>;
    fn concat_legacy(&self) -> Vec<u8>;
}
pub trait ConcatLegacy2{
    fn concat(&self) -> Vec<u8>;
    fn concat_legacy(&self, index: usize) -> Vec<u8>;
}
pub trait ConcatEmpty{
    fn concat(&self) -> Vec<u8>;
    fn concat_empty(&self, index: usize) -> Vec<u8>;
}
    
impl ConcatLegacy2 for Vec<Vin>{
    fn concat_legacy(&self, index: usize) -> Vec<u8> {
        let mut concatenated_bytes: Vec<u8> = Vec::new();
        for (j, vin) in self.iter().enumerate(){
            if j == index {
            concatenated_bytes.append(&mut vin.clone().concat_legacy());
            }else{
            concatenated_bytes.append(&mut vin.clone().concat_legacy_empty());
            }
        }
        concatenated_bytes
    }
    fn concat(&self) -> Vec<u8> {
        let mut concatenated_bytes: Vec<u8> = Vec::new();
        for vin in self{
            concatenated_bytes.append(&mut vin.clone().concat())
        }
        concatenated_bytes
    }
}
impl ConcatLegacy for Vec<StackItem>{
    fn concat_legacy(&self) -> Vec<u8> {
        let mut concatenated_bytes: Vec<u8> = Vec::new();
        for stack_item in self{
            concatenated_bytes.append(&mut stack_item.clone().concat_legacy())
        }
        concatenated_bytes
    }
    fn concat(&self) -> Vec<u8> {
        let mut concatenated_bytes: Vec<u8> = Vec::new();
        for stack_item in self{
            concatenated_bytes.append(&mut stack_item.clone().concat())
        }
        concatenated_bytes
    }
}
impl ConcatEmpty for Vec<Vout>{
    fn concat(&self) -> Vec<u8> {
        let mut concatenated_bytes: Vec<u8> = Vec::new();
        for vout in self{
            concatenated_bytes.append(&mut vout.clone().concat())
        }
        concatenated_bytes
    }
    fn concat_empty(&self, index: usize) -> Vec<u8> {
        let mut concatenated_bytes: Vec<u8> = Vec::new();
        for (j, vout )in self.into_iter().enumerate(){
            if index == j{
                concatenated_bytes.append(&mut vout.clone().concat());
            }else{
                concatenated_bytes.append(&mut vec![0]);
            }
        }
        concatenated_bytes
    }
}
impl Concat for Vec<Option<Witness>>{
    fn concat(&self) -> Vec<u8> {
        let mut concatenated_bytes: Vec<u8> = Vec::new();
        for witness in self{
            concatenated_bytes.append(&mut match witness{
                Some(x) => x.clone().concat(),
                None => vec![0u8]
            })
        }
        concatenated_bytes
    }
}
impl Concat for Vec<Signature>{
    fn concat(&self) -> Vec<u8> {
        let mut concatenated_bytes: Vec<u8> = Vec::new();
        for signature in self{
            concatenated_bytes.append(&mut signature.clone().concat())
        }
        concatenated_bytes
    }
}
impl Concat for Vec<VinLegacy>{
    fn concat(&self) -> Vec<u8> {
        let mut concatenated_bytes: Vec<u8> = Vec::new();
        for vin in self{
            concatenated_bytes.append(&mut vin.clone().concat())
        }
        concatenated_bytes
    }
}
impl Concat for Vec<VinSigned>{
    fn concat(&self) -> Vec<u8> {
        let mut concatenated_bytes: Vec<u8> = Vec::new();
        for vin_signed in self{
            concatenated_bytes.append(&mut vin_signed.clone().concat())
        }
        concatenated_bytes
    }
}












//LEGACY

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedTransactionLegacy {
    pub version: [u8;4],
    pub vin_count: Vec<u8>,
    pub vins: Vec<VinLegacy>,
    pub vout_count: Vec<u8>,
    pub vouts: Vec<Vout>,
    pub locktime: [u8;4],
    pub sighash_type: [u8;4],
}

    impl UnsignedTransactionLegacy{
        pub fn new(version: u32, vins: Vec<VinLegacy>, vouts: Vec<Vout>, locktime: u32) -> Self{
            UnsignedTransactionLegacy{
                version: version.to_le_bytes(),
                vin_count: varint(vins.len()),
                vins,
                vout_count: varint(vouts.len()),
                vouts,
                locktime: locktime.to_le_bytes(),
                sighash_type: [1,0,0,0]
            }
        }

        pub fn concat(self) -> Vec<u8> {
            self.version.into_iter()
            .chain(self.vin_count.into_iter())
            .chain(self.vins.concat().into_iter())
            .chain(self.vout_count.into_iter())
            .chain(self.vouts.concat().into_iter())
            .chain(self.locktime.into_iter())
            .chain(self.sighash_type.into_iter())
            .collect()
        }
    }
    
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VinLegacy {
    pub txid: Vec<u8>,
    pub vout: [u8;4],
    pub locking_script_length: Vec<u8>,
    pub locking_script: Vec<u8>,
    pub sequence: [u8;4],
}

    impl VinLegacy{
        pub fn new(txid: &str, vout: u32, locking_script: &str, sequence: u32) -> Result<Self, String>{
            Ok(VinLegacy{
                txid: txid.to_bytes()?.reverse(),
                vout: vout.to_le_bytes(),
                locking_script_length: varint(locking_script.len()/2),
                locking_script: locking_script.to_bytes()?,
                sequence: sequence.to_le_bytes(),
            })
        }
        
        pub fn concat(self) -> Vec<u8> {
            self.txid.into_iter()
            .chain(self.vout.into_iter())
            .chain(self.locking_script_length.into_iter())
            .chain(self.locking_script.into_iter())
            .chain(self.sequence.into_iter())
            .collect()
        }
    }

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VinSigned {
    pub txid: Vec<u8>,
    pub vout: [u8;4],
    pub script_sig_length: Vec<u8>,
    pub signature_length: Vec<u8>,
    pub signature: Vec<u8>,
    pub sighash_type: [u8;1],
    pub redeem_script_length: Vec<u8>,
    pub redeem_script: Vec<u8>,
    pub sequence: [u8;4],
}

    impl VinSigned{
        pub fn new(vin: Vin, signature: &str, redeem_script: &str) -> Result<Self, String>{
            let signature_length = varint(signature.len()/2);
            let signature = signature.to_bytes()?;
            let redeem_script_length = varint(redeem_script.len()/2);
            let redeem_script = redeem_script.to_bytes()?;
            let script_sig_length = varint(signature_length.len() + signature.len() + 1 + redeem_script_length.len() + redeem_script.len());

            Ok(VinSigned{
                txid: vin.txid,
                vout: vin.vout,
                script_sig_length,
                signature_length,
                signature,
                sighash_type: [1],
                redeem_script_length,
                redeem_script,
                sequence: vin.sequence,
            })
        }

        pub fn concat(self) -> Vec<u8> {
            self.txid.into_iter()
            .chain(self.vout.into_iter())
            .chain(self.script_sig_length.into_iter())
            .chain(self.signature_length.into_iter())
            .chain(self.signature.into_iter())
            .chain(self.sighash_type.into_iter())
            .chain(self.redeem_script_length.into_iter())
            .chain(self.redeem_script.into_iter())
            .chain(self.sequence.into_iter())
            .collect()
        }
    }


fn script (script: &str) -> Option<&str>{
    Some(match script.to_ascii_uppercase().as_ref() {
        "OP_FALSE" => "00",
        "OP_0" => "00",
        "OP_PUSHDATA1" => "4c",
        "OP_PUSHDATA2" => "4d",
        "OP_PUSHDATA4" => "4e",
        "OP_1NEGATE" => "4f",
        "OP_TRUE" => "51",
        "OP_1" => "51",
        "OP_2" => "52",
        "OP_3" => "53",
        "OP_4" => "54",
        "OP_5" => "55",
        "OP_6" => "56",
        "OP_7" => "57",
        "OP_8" => "58",
        "OP_9" => "59",
        "OP_10" => "5a",
        "OP_11" => "5b",
        "OP_12" => "5c",
        "OP_13" => "5d",
        "OP_14" => "5e",
        "OP_15" => "5f",
        "OP_16" => "60",
        "OP_NOP" => "61",
        "OP_IF" => "63",
        "OP_NOTIF" => "64",
        "OP_ELSE" => "67",
        "OP_ENDIF" => "68",
        "OP_VERIFY" => "69",
        "OP_RETURN" => "6a",
        "OP_TOALTSTACK" => "6b",
        "OP_FROMALTSTACK" => "6c",
        "OP_IFDUP" => "73",
        "OP_DEPTH" => "74",
        "OP_DROP" => "75",
        "OP_DUP" => "76",
        "OP_NIP" => "77",
        "OP_OVER" => "78",
        "OP_PICK" => "79",
        "OP_ROLL" => "7a",
        "OP_ROT" => "7b",
        "OP_SWAP" => "7c",
        "OP_TUCK" => "7d",
        "OP_2DROP" => "6d",
        "OP_2DUP" => "6e",
        "OP_3DUP" => "6f",
        "OP_2OVER" => "70",
        "OP_2ROT" => "71",
        "OP_2SWAP" => "72",
        "OP_SIZE" => "82",
        "OP_EQUAL" => "87",
        "OP_EQUALVERIFY" => "88",
        "OP_1ADD" => "8b",
        "OP_1SUB" => "8c",
        "OP_NEGATE" => "8f",
        "OP_ABS" => "90",
        "OP_NOT" => "91",
        "OP_0NOTEQUAL" => "92",
        "OP_ADD" => "93",
        "OP_SUB" => "94",
        "OP_BOOLAND" => "9a",
        "OP_BOOLOR" => "9b",
        "OP_NUMEQUAL" => "9c",
        "OP_NUMEQUALVERIFY" => "9d",
        "OP_NUMNOTEQUAL" => "9e",
        "OP_LESSTHAN" => "9f",
        "OP_GREATERTHAN" => "a0",
        "OP_LESSTHANOREQUAL" => "a1",
        "OP_GREATERTHANOREQUAL" => "a2",
        "OP_MIN" => "a3",
        "OP_MAX" => "a4",
        "OP_WITHIN" => "a5",
        "OP_RIPEMD160" => "a6",
        "OP_SHA1" => "a7",
        "OP_SHA256" => "a8",
        "OP_HASH160" => "a9",
        "OP_HASH256" => "aa",
        "OP_CODESEPARATOR" => "ab",
        "OP_CHECKSIG" => "ac",
        "OP_CHECKSIGVERIFY" => "ad",
        "OP_CHECKMULTISIG" => "ae",
        "OP_CHECKMULTISIGVERIFY" => "af",
        "OP_CHECKLOCKTIMEVERIFY" => "b1",
        "OP_CHECKSEQUENCEVERIFY" => "b2",
        _ => return None,
    })
}