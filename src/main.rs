#[macro_use]
extern crate serde_derive;

use iced::theme;
use iced::widget::{container, horizontal_space, vertical_space};
use iced::{Color, Element, Length, Sandbox, Settings};
use iced::widget::{self, button, row, column, text, text_input, scrollable, checkbox, pick_list};
use iced::alignment::{self, Alignment};
use secp256k1::rand::rngs::*;
use secp256k1::Secp256k1;
use secp256k1::*;
use sha2::{Sha256, Digest};

use std::str::FromStr;

use numeric_input::numeric_input;
use crate::write_file::write_file;
use number_input::number_input;
use number_input_1::number_input_1;
use number_input_2::number_input_2;

mod traits;
mod wallet_info;
mod file;
mod blockchain_info;
mod blockchain_status;
mod blockchain_address;
mod blockchain_transaction;
mod blockchain_utxo;
mod create_transaction;
mod transaction_parts;
mod will_components;

use crate::blockchain_info::*;
use crate::traits::*;
use crate::wallet_info::*;
use crate::file::*;
use crate::blockchain_status::BlockchainStatus;
use crate::blockchain_address::BlockchainAddress;
use crate::blockchain_transaction::BlockchainTransaction;
use crate::blockchain_utxo::UTXO;
use crate::create_transaction::*;
use crate::transaction_parts::*;
use crate::will_components::*;


fn main() -> iced::Result {
    App::run(Settings::default())
}

struct App{
    info: WalletInfo,
    launch: Launch,
    set_will: SetWill,
    transaction_history: ViewTransactionHistory,
    send_transaction: SendTransaction,
}

impl Sandbox for App{
    type Message = Message;

    fn new() -> Self{
        App {
            info: WalletInfo::new_empty(),
            launch: Launch::new(),
            set_will: SetWill::new(),
            transaction_history: ViewTransactionHistory::new(),
            send_transaction: SendTransaction::new(),
        }
    }

    fn title(&self) -> String{
        String::from("ION Wallet")
    }

    fn view(&self) -> Element<Message>{
        let mut master = row![];

        if self.launch.locked(){
            master = master.push(self.launch.view());
        }else{
            let balance = row![
                text(format!("Balance Total: {}", self.info.value/100000000)).size(70),
                container(column![
                    vertical_space(Length::Units(25)),
                    text(sat_decimal(self.info.value)).size(38),
                ]),
                text(" BTC").size(70),
            ];
            
            let mut inheritors_list: String = String::new();
            for (i,inheritor) in self.info.inheritors.iter().enumerate(){
                inheritors_list.push_str(&format!("Inheritor{}: {}\n", i+1, inheritor.address.shorten(14)));
                inheritors_list.push_str(&format!("{} BTC\n\n", inheritor.value as f64 / 100000000.0));
            }

            let redemption_period = column![
                text("Will Redemption Period:").size(25),
                text(format!("{} Blocks (≈{} Days)", self.info.locktime, self.info.locktime/144)),
            ].spacing(10);
            
            let info_column = column![
                text(format!("Address:\n{}", self.info.address)).size(25),
                text(inheritors_list).size(25),
                redemption_period,
            ].spacing(40).width(Length::Units(400));

            let buttons = column![
                vertical_space(Length::Units(40)),
                button("Update").on_press(Message::Update),
                button("Send Coins").on_press(Message::OpenSendTransaction),
                button("Set Will / Change Will").on_press(Message::OpenSetWill),
                button("View Transaction History").on_press(Message::OpenHistory),
            ].spacing(30).width(Length::Units(260)).align_items(Alignment::End);
            
            let double = row![
                info_column,
                horizontal_space(Length::Fill),
                buttons,
            ].width(Length::Units(710)).spacing(10);
            

            let main_content = column![
                container(balance).width(Length::Units(740)).center_x(),
                double,
            ].height(Length::Units(800)).spacing(1);

            let main = container(main_content).width(Length::Units(740)).center_y();

            master = master.push(main);
            
            if self.set_will.is_on(){
                master = master.push(self.set_will.view().map(Message::SetWillMessage))
            }
            if self.transaction_history.is_on(){
                master = master.push(self.transaction_history.view())
            }
            if self.send_transaction.is_on(){
                master = master.push(self.send_transaction.view().map(Message::TransactionMessage))
            }

        }
        let master_container = container(master).width(Length::Units(1950)).padding(50).center_x().center_y();
        
        let total: Element<_> = row![container(master_container)].into();

        total
    }

    fn update(&mut self, message: Message){
        match message{
            Message::OpenSetWill => {
                self.set_will.on()
            }
            Message::SetWillMessage(x) => {
                match x {
                    SetWillMessage::Finish => {
                        self.info.inheritors = self.set_will.inheritors.clone();
                        self.info.guardians = self.set_will.guardians.clone();
                        self.info.locktime = self.set_will.pages.get_locktime_blocks();
                        write_wallet(self.info.clone(), &self.launch.password);
                        self.set_will.create_will(&mut self.info);
                        self.set_will.update(x)
                    }
                    _ => {
                        self.set_will.update(x)
                    }
                }
            }
            Message::TypePassword(x) => {
                self.launch.update_password(x);
            }
            Message::EnterPassword => {
                match self.launch.enter_password(){
                    Ok(x) => self.info = x,
                    _ => ()
                };
            }
            Message::Update => {
                let address = blockchain_info::testnet_address_request(&self.info.address);
                self.info.value = address.balance.parse::<u64>().unwrap();
                write_wallet(self.info.clone(), &self.launch.password);
                write_transaction_history(blockchain_info::testnet_address_history(&self.info.address, 6), &self.launch.password);
                self.transaction_history.set(read_transaction_history(&self.launch.password));
            }
            Message::OpenHistory => {
                self.transaction_history.on();
                if self.transaction_history.is_empty(){
                    self.transaction_history.set(read_transaction_history(&self.launch.password));
                };
            }
            Message::CloseHistory => {
                self.transaction_history.off();
            }
            Message::TransactionMessage(x) => {
                match x{
                    TransactionMessage::Create => {
                        self.send_transaction.create_transaction(&mut self.info);
                        self.send_transaction.update(x)
                    }
                    _ => {
                        self.send_transaction.update(x)
                    }
                }
            }
            Message::OpenSendTransaction => {
                self.send_transaction.on()
            }
        }
    }
}


#[derive(Debug, Clone)]
enum Message{
    OpenSetWill,
    SetWillMessage(SetWillMessage),
    TypePassword(String),
    EnterPassword,
    Update,
    OpenHistory,
    CloseHistory,
    TransactionMessage(TransactionMessage),
    OpenSendTransaction,
}

struct SendTransaction{
    signed_transaction: Option<SignedTransaction>,
    signed_transaction_string: Option<String>,
    address: String,
    value: u64,
    fee: u64,
    on: bool,
    password: String,
    enter_password: bool,
    debug: String,
}

impl SendTransaction{

    fn new() -> Self{
        SendTransaction{
            signed_transaction: None,
            signed_transaction_string: None,
            address: String::new(),
            value: 0,
            fee: 0,
            on: false,
            password: String::new(),
            enter_password: false,
            debug: String::new(),
        }
    }

    fn view(&self) -> Element<TransactionMessage>{
        let mut contents = column![
            row![horizontal_space(Length::Fill), button("x").on_press(TransactionMessage::Close)],
        ].width(Length::Units(640)).align_items(Alignment::Start).height(Length::Units(750)).spacing(25);

        contents = contents.push(text_input("Address:", &self.address, TransactionMessage::SetAddress));
        contents = contents.push(number_input(self.value, TransactionMessage::SetValue));
        contents = contents.push(number_input(self.fee, TransactionMessage::SetFee));
        contents = contents.push(button("Create Transaction (Unsigned)").on_press(TransactionMessage::EnterInfo));

        if self.enter_password{
            contents = contents.push(text_input("Password:", &self.password, TransactionMessage::EnterPassword));
            contents = contents.push(button("Sign Transaction").on_press(TransactionMessage::Create));
        }
        contents = contents.push(button("Save/Print Transaction").on_press(TransactionMessage::Save));
        contents = contents.push(button("Broadcast Transaction").on_press(TransactionMessage::Broadcast));
        contents = contents.push(text(&self.debug));
        contents = contents.push(vertical_space(Length::Fill));
        column![container(contents).height(Length::Fill).center_x().center_y()].into()
    }

    fn update(&mut self, message: TransactionMessage){
        match message{
            TransactionMessage::Save => {
                self.update_debug(
                    self.signed_transaction_string.clone()
                    .unwrap_or(String::from("No Transaction Created"))
                )
            }
            TransactionMessage::SetAddress(x) => {self.address = x}
            TransactionMessage::SetValue(x) => {self.value = x}
            TransactionMessage::SetFee(x) => {self.fee = x}
            TransactionMessage::EnterInfo => {self.enter_password = true}
            TransactionMessage::EnterPassword(x) => {self.password = x}
            TransactionMessage::Create => {
                self.password = String::new();
                self.enter_password = false;
            }
            TransactionMessage::Close => {self.on = false}
            TransactionMessage::Broadcast => {
                let transaction_text = match self.signed_transaction_string.clone(){
                    Some(n) => testnet_broadcast_transaction(&n),
                    None => String::from("No Transaction Created")
                };
                self.update_debug(transaction_text)
            }
        }
    }

    fn on(&mut self){
        self.on = true
    }
    fn is_on(&self) -> bool{
        self.on
    }
    fn create_transaction(&mut self, will_info: &mut WalletInfo){
        let shrink_factor: f64 = 1.0 - ((self.value + self.fee) as f64 / will_info.value as f64);
        let secretkey = SecretKey::from_str(&read_keys(&self.password)).unwrap();
        self.signed_transaction = create_transaction(&self.address, self.value, self.fee, &will_info.address, &will_info.pubkey, secretkey).ok();
        self.signed_transaction_string = match self.signed_transaction.clone(){
            Some(n) => Some(n.concat().to_string()),
            None => None
        };
        for i in 0..will_info.inheritors.len(){
            will_info.inheritors[i].value = (will_info.inheritors[i].value as f64 * shrink_factor) as u64;
        }
        let will_parts = predict_will_parts(will_info.inheritors.addresses(), will_info.inheritors.amounts(), will_info.locktime as u16, self.signed_transaction.clone().unwrap(), &(*will_info.address), &(*will_info.pubkey), secretkey);
        write_will_parts(will_parts);
        
        write_wallet(will_info.clone(), &self.password);
    }
    fn update_debug(&mut self, text: String){
        self.debug = text + "\n\n" + &self.debug
    }
}

#[derive(Debug, Clone)]
enum TransactionMessage{
    Save,
    SetAddress(String),
    SetValue(u64),
    SetFee(u64),
    EnterInfo,
    Create,
    Close,
    EnterPassword(String),
    Broadcast,
}

struct Launch{
    password: String,
    unlocked: bool,
    wrong_password: bool,
}

impl Launch{
    fn new() -> Self{
        Launch {
            password: String::new(),
            unlocked: false,
            wrong_password: false,
        }
    }

    fn view(&self) -> Element<Message>{
        let mut contents = column![
            text("Password").size(40),
            text_input("-Enter Password-", &self.password, Message::TypePassword).on_submit(Message::EnterPassword)
        ].width(Length::Units(450)).align_items(Alignment::Center);

        if self.wrong_password{
            contents = contents.push(text("Wrong Password"))
        }

        column![container(contents).height(Length::Fill).center_x().center_y()].into()
    }

    fn update_password(&mut self, password: String){
        self.password = password;
    }

    fn enter_password(&mut self) -> Result<WalletInfo, String>{
        self.wrong_password = false;
        let wallet = match read_wallet(&self.password){
            Ok(x) => x,
            Err(e) => {
                self.wrong_password = true;
                return Err(e);
            }
        };
        self.unlocked = true;
        Ok(wallet)
    }

    fn locked(&self) -> bool{
        !self.unlocked
    }
}


struct ViewTransactionHistory{
    history: Vec<TransactionHistory>,
    on: bool,
}

impl ViewTransactionHistory{
    fn new() -> Self{
        ViewTransactionHistory {
            history: vec![],
            on: false,
        }
    }

    fn view(&self) -> Element<Message>{
        let mut contents = column![
            row![horizontal_space(Length::Fill), button("x").on_press(Message::CloseHistory)],
        ].width(Length::Units(640)).align_items(Alignment::Start).height(Length::Units(750)).spacing(25);

        for transaction in self.history.clone(){
            let mut to_address = row![].align_items(Alignment::End);
            if transaction.to{
                to_address = to_address.push(text("Sent To -> "));
                to_address = to_address.push(text(transaction.address).size(12));
            }else{
                to_address = to_address.push(text("Received From: "));
                to_address = to_address.push(text(transaction.address).size(12));
            }
            contents = contents.push(column![
                to_address,
                text(&format!("Value: {} sats     {} CONFIRMATIONS", transaction.value, transaction.confirmations))
            ]);
        }

        contents = contents.push(vertical_space(Length::Fill));

        column![container(contents).height(Length::Fill).center_x().center_y()].into()
    }

    fn set(&mut self, history: Result<Vec<TransactionHistory>, String>){
        match history{
            Ok(x) => self.history = x,
            _ => ()
        };
    }

    fn is_empty(&self) -> bool{
        self.history.len() == 0
    }
    
    fn is_on(&self) -> bool {
        self.on
    }
    fn on(&mut self){
        self.on = true;
    }
    fn off(&mut self){
        self.on = false;
    }
}


struct SetWill{
    pages: Pages,
    inheritors: Vec<Inheritor>,
    guardians: Vec<Guardian>,
    locktime: u64,
    on: bool,
    password: String,
}
impl Sandbox for SetWill{
    type Message = SetWillMessage;

    fn new() -> SetWill{
        SetWill {
            pages: Pages::new(),
            inheritors: vec![],
            guardians: vec![],
            locktime: 0,
            on: false,
            password: String::new(),
        }
    }

    fn title(&self) -> String{
        String::from("Set A Number")
    }

    fn view(&self) -> Element<SetWillMessage>{
        let SetWill { pages, .. } = self;

        let mut controls_top = row![
            horizontal_space(Length::Fill),
            button("x").on_press(SetWillMessage::Close)
        ];
        let mut controls_bottom = row![];

        if self.pages.has_previous(){
            controls_bottom = controls_bottom.push(
                button("Back")
                .on_press(SetWillMessage::Back)
            )
        }

        if self.pages.has_next(){
            controls_bottom = controls_bottom.push(horizontal_space(Length::Fill));
            
            if pages.next_is_inheritors(){
                controls_bottom = controls_bottom.push(
                    button("Next")
                    .on_press(SetWillMessage::AddInheritors(self.pages.n_inheritors().try_into().unwrap()))
                )
            }else if pages.is_inheritor(){
                controls_bottom = controls_bottom.push(
                    button("Next")
                    .on_press(SetWillMessage::NextInheritor)
                )
            }else if pages.next_is_guardians(){
                controls_bottom = controls_bottom.push(
                    button("Next")
                    .on_press(SetWillMessage::AddGuardians(self.pages.n_guardians().try_into().unwrap()))
                )
            }else if pages.is_guardian(){
                controls_bottom = controls_bottom.push(
                    button("Next")
                    .on_press(SetWillMessage::NextGuardian)
                )
            }else{
                controls_bottom = controls_bottom.push(
                    button("Next")
                    .on_press(SetWillMessage::Next)
                )
            }
        }else{
            controls_bottom = controls_bottom.push(column![
                text_input("-Enter Password-", &self.password, SetWillMessage::EnterPassword).width(Length::Units(200))
            ].width(Length::Fill).align_items(Alignment::Center));
            controls_bottom = controls_bottom.push(
                button("Finish")
                .on_press(SetWillMessage::Finish)
            )
        }

        let mut inheritors_list: String = String::new();
        let mut guardians_list: String = String::new();
        if pages.exit_inheritors(){
            for (i,inheritor) in self.inheritors.iter().enumerate(){
                inheritors_list.push_str(&format!("Inheritor {}\n", i+1));
                inheritors_list.push_str(&format!("Name: {}\n", inheritor.name));
                inheritors_list.push_str(&format!("Address: {}\n", inheritor.address));
                inheritors_list.push_str(&format!("User ID: {}\n", inheritor.id));
                inheritors_list.push_str(&format!("Amount (Satoshis): {}\n", inheritor.value));
                inheritors_list.push_str(&format!("\n\n"));
            }
        }else if pages.exit_guardians(){
            for (i,guardian) in self.guardians.iter().enumerate(){
                guardians_list.push_str(&format!("Guardian {}\n", i+1));
                guardians_list.push_str(&format!("Name: {}\n", guardian.name));
                guardians_list.push_str(&format!("User ID: {}\n", guardian.id));
                guardians_list.push_str(&format!("\n\n"));
            }
        }

        let content: Element<_> = column![
            controls_top,
            self.pages.view(inheritors_list, guardians_list).map(SetWillMessage::PagesMessages),
            vertical_space(Length::Fill),
            controls_bottom,
        ]
        .height(Length::Units(650))
        .max_width(800)
        .spacing(10)
        .padding(10)
        .align_items(Alignment::Center)
        .into();

        let scrollable = scrollable(
            container(content)
            .width(Length::Fill)
            .center_x()
        );

        container(scrollable).height(Length::Fill).center_x().into()
    }

    fn update(&mut self, message: SetWillMessage){
        match message{
            SetWillMessage::Next => self.pages.next(),
            SetWillMessage::Back => self.pages.back(),
            SetWillMessage::PagesMessages(val) => self.pages.update(val),
            SetWillMessage::AddInheritors(n) => {self.pages.add_inheritors(); self.pages.next()}
            SetWillMessage::NextInheritor => {
                let inheritor = self.pages.current_inheritor_info();
                if self.pages.inheritor_is_guardian(){
                    self.guardians.push(Guardian { name: (inheritor.name.clone()), id: (inheritor.id.clone()) });
                }
                self.inheritors.push(inheritor);
                self.pages.next()}
            SetWillMessage::AddGuardians(n) => {self.pages.add_guardians(); self.pages.next()}
            SetWillMessage::NextGuardian => {self.guardians.push(self.pages.current_guardian_info()); self.pages.next()}
            SetWillMessage::EnterPassword(x) => {self.password = x}
            SetWillMessage::Finish => {self.password = String::new(); self.on = false}
            SetWillMessage::Close => {self.password = String::new(); self.on = false}
        }
    }
}
impl SetWill{
    fn is_on(&self) -> bool {
        self.on
    }
    fn on(&mut self){
        self.on = true;
    }
    fn create_will(&mut self, will_info: &mut WalletInfo){
        let secretkey = SecretKey::from_str(&read_keys(&self.password)).unwrap();
        let will_parts = create_will_parts(will_info.inheritors.addresses(), will_info.inheritors.amounts(), will_info.locktime as u16, &(*will_info.address), &(*will_info.pubkey), secretkey);
        write_will_parts(will_parts);
    }
}



#[derive(Debug, Clone)]
enum SetWillMessage{
    Next,
    Back,
    PagesMessages(PageMessage),
    AddInheritors(u8),
    NextInheritor,
    AddGuardians(u8),
    NextGuardian,
    EnterPassword(String),
    Finish,
    Close,
}
#[derive(Debug, Clone)]
enum PageMessage{
    ChangeInputInheritors(Option<u8>),
    InheritorMessages(InheritorMessage),
    ChangeInputGuardians(Option<u8>),
    GuardianMessages(GuardianMessage),
    SetLocktime(u32),
    SetTimeUnit(TimeUnit),
}
#[derive(Debug, Clone)]
enum InheritorMessage{
    SetName(String),
    SetAddress(String),
    SetID(String),
    SetValue(u64),
    ToggleGuardian(bool),
}
#[derive(Debug, Clone)]
enum GuardianMessage{
    SetName(String),
    SetID(String),
}


struct Pages{
    pages: Vec<Page>,
    current: usize,
    n_inheritors: Option<u8>,
    n_guardians: Option<u8>,
}
impl Pages{
    fn new() -> Self{
        Pages{
            pages: vec![
                Page::SetNumberOfInheritors,
                Page::ShowInheritors,
                Page::SetNumberOfGuardians,
                Page::ShowGuardians,
                Page::SetLocktime(SetLocktime::new()),
            ],
        current: 0,
        n_inheritors: Some(1),
        n_guardians: None,
        }
    }

    fn update(&mut self, message: PageMessage){
        self.pages[self.current].update(message, &mut self.n_inheritors, &mut self.n_guardians)
    }

    fn view(&self, inheritors: String, guardians: String) -> Element<PageMessage>{
        self.pages[self.current].view(self.n_inheritors, self.n_guardians, self.current, inheritors, guardians)
    }

    fn back(&mut self){
        if self.current > 0{
            self.current -= 1
        }
    }

    fn next(&mut self){
        if self.current < self.pages.len()-1{
            self.current += 1
        }
    }

    fn has_next(&self) -> bool{
        self.current < self.pages.len()-1
    }

    fn has_previous(&self) -> bool{
        self.current > 0
    }

    fn next_is_inheritors(&self) -> bool{
        self.current == 0
    }

    fn is_inheritor(&self) -> bool{
        self.current > 0 && self.current < 1 + self.n_inheritors()
    }

    fn add_inheritors(&mut self){
        for i in 0..self.n_inheritors(){
            self.pages.insert(1, Page::NewInheritor(NewInheritor::new(i.try_into().unwrap())));
        }
    }
    
    fn current_inheritor_info(&mut self) -> Inheritor{
        self.pages[self.current].inheritor_info()
    }

    fn inheritor_is_guardian(&self) -> bool{
        self.pages[self.current].inheritor_is_guardian()
    }

    fn exit_inheritors(&self) -> bool{
        self.current == 1 + self.n_inheritors()
    }
    
    fn next_is_guardians(&self) -> bool{
        self.current == 2 + self.n_inheritors()
    }

    fn is_guardian(&self) -> bool{
        self.current > 2 + self.n_inheritors() && self.current < 3 + self.n_inheritors() + self.n_guardians()
    }

    fn add_guardians(&mut self){
        for i in 0..self.n_guardians(){
            self.pages.insert(3 + self.n_inheritors() as usize, Page::NewGuardian(NewGuardian::new(i.try_into().unwrap())));
        }
    }
    
    fn current_guardian_info(&mut self) -> Guardian{
        self.pages[self.current].guardian_info()
    }

    fn exit_guardians(&self) -> bool{
        self.current == 3 + self.n_inheritors() + self.n_guardians()
    }

    fn print(&self){
        println!("{:?}", self.pages.clone())
    }
    
    fn n_inheritors(&self) -> usize{
        self.n_inheritors.unwrap_or(1).into()
    }

    fn n_guardians(&self) -> usize{
        self.n_guardians.unwrap_or(0).into()
    }

    fn get_locktime_blocks(&self) -> u32{
        self.pages[self.current].get_locktime_blocks()
    }

}


#[derive(Debug, Clone)]
enum Page{
    SetNumberOfInheritors,
    NewInheritor(NewInheritor),
    ShowInheritors,
    SetNumberOfGuardians,
    NewGuardian(NewGuardian),
    ShowGuardians,
    SetLocktime(SetLocktime),
}
impl Page{

    fn view(&self, n_inheritors: Option<u8>, n_guardians: Option<u8>, current: usize, inheritors: String,
        guardians: String) -> Element<PageMessage>{
        match self{
            Page::SetNumberOfInheritors => {
                column![
                    text("How Many Inheritors?").size(50),
                    vertical_space(Length::Units(140)),
                    numeric_input(n_inheritors, PageMessage::ChangeInputInheritors),
                ]
                .max_width(800)
                .spacing(10)
                .padding(10)
                .align_items(Alignment::Center)
                .into()
            }
            Page::NewInheritor(x) => {
                let i = current;
                let content: Element<_> = column![
                    x.view(i).map(PageMessage::InheritorMessages),
                ]
                .max_width(800)
                .spacing(10)
                .padding(10)
                .align_items(Alignment::Center)
                .into();
                container(content).center_x().into()
            }
            Page::ShowInheritors => {
                let content= column![
                    text(&inheritors).size(35),
                ]
                .max_width(800)
                .spacing(10)
                .padding(10)
                .align_items(Alignment::Center);
                
                let scrollable = scrollable(
                    container(content)
                    .width(Length::Fill)
                    .center_x()
                );

                container(column![
                    text("Please verify your Inheritors:").size(20),
                    scrollable
                ]
                .spacing(20))
                .height(Length::Fill)
                .height(Length::Units(500))
                .center_x()
                .into()
            }
            Page::SetNumberOfGuardians => {
                column![
                    text("How Many Guardians?").size(50),
                    vertical_space(Length::Units(140)),
                    numeric_input(n_guardians, PageMessage::ChangeInputGuardians),
                ]
                .max_width(800)
                .spacing(10)
                .padding(10)
                .align_items(Alignment::Center)
                .into()
            }
            Page::NewGuardian(x) => {
                let i = current - 2 - n_inheritors.unwrap_or(1) as usize;
                let content: Element<_> = column![
                    x.view(i).map(PageMessage::GuardianMessages),
                ]
                .max_width(800)
                .spacing(10)
                .padding(10)
                .align_items(Alignment::Center)
                .into();
                container(content).center_x().into()
            }
            Page::ShowGuardians => {
                let content = column![
                    text(&guardians).size(35),
                ]
                .max_width(800)
                .spacing(10)
                .padding(10)
                .align_items(Alignment::Center);

                let scrollable = scrollable(
                    container(content)
                    .width(Length::Fill)
                    .center_x()
                );

                container(column![
                    text("Please verify your Guardians:").size(20),
                    scrollable
                ]
                .spacing(20))
                .height(Length::Fill)
                .height(Length::Units(500))
                .center_x()
                .into()
            }
            Page::SetLocktime(x) => {
                let content: Element<_> = column![
                    x.view(),
                ]
                .max_width(800)
                .spacing(10)
                .padding(10)
                .align_items(Alignment::Center)
                .into();
                container(content).center_x().into()
            }
        }
    }

    fn update(&mut self, message: PageMessage, n_inheritors: &mut Option<u8>, n_guardians: &mut Option<u8>){
        match message{
            PageMessage::ChangeInputInheritors(val) => {
                *n_inheritors = val
            }
            PageMessage::InheritorMessages(val) => {
                if let  Page::NewInheritor(x) = self{
                   x.update(val)
                }
            }
            PageMessage::ChangeInputGuardians(val) => {
                *n_guardians = val
            }
            PageMessage::GuardianMessages(val) => {
                if let Page::NewGuardian(x) = self {
                    x.update(val)
                }
            }
            PageMessage::SetLocktime(val) => {
                if let Page::SetLocktime(x) = self {
                    x.set_value(val)
                }
            }
            PageMessage::SetTimeUnit(val) => {
                if let Page::SetLocktime(x) = self {
                    x.set_unit(val)
                }
            }
        }
    }

    fn inheritor_info(&mut self) -> Inheritor{
        match self{
            Page::NewInheritor(x) => {
                x.info()
            }
            _ => panic!["Cannot Get Inheritor Info"]
        }
    }
    fn guardian_info(&mut self) -> Guardian{
        match self{
            Page::NewGuardian(x) => {
                x.info()
            }
            _ => panic!["Cannot Get Guardian Info"]
        }
    }
    fn inheritor_is_guardian(&self) -> bool{
        if let Page::NewInheritor(x) = self{
            x.is_guardian
        }else{false}
    }
    fn get_locktime_blocks(&self) -> u32{
        if let Page::SetLocktime(x) = self{
            x.get_locktime_blocks()
        }else{4294967295}
    }
}


#[derive(Debug, Clone)]
struct NewInheritor{
    nth: u8,
    name: String,
    address: String,
    id: String,
    value: u64,
    is_guardian: bool,
}
impl NewInheritor{
    fn new(nth: u8) -> Self{
        NewInheritor{
            nth,
            name: String::from(""),
            address: String::from(""),
            id: String::from(""),
            value: 0,
            is_guardian: false,
        }
    }

    fn view(&self, i: usize) -> Element<InheritorMessage>{
        return column![
            text(format!("Inheritor: {}", i)).size(50),
            text_input("Name:", &self.name, InheritorMessage::SetName),
            text_input("Address:", &self.address, InheritorMessage::SetAddress),
            text_input("UserID:", &self.id, InheritorMessage::SetID),
            number_input(self.value, InheritorMessage::SetValue),
            checkbox("Include as Guardian", self.is_guardian, InheritorMessage::ToggleGuardian),
        ]
        .max_width(800)
        .spacing(10)
        .padding(10)
        .align_items(Alignment::Center)
        .into();
    }

    fn update(&mut self, message: InheritorMessage){
        match message{
            InheritorMessage::SetName(x) => self.name = x,
            InheritorMessage::SetAddress(x) => self.address = x,
            InheritorMessage::SetID(x) => self.id = x,
            InheritorMessage::SetValue(x) => self.value = x,
            InheritorMessage::ToggleGuardian(x) => self.is_guardian = x,
        }
    }

    fn info(&mut self) -> Inheritor{
        let inheritor = Inheritor{
            name: self.name.clone(),
            address: self.address.clone(),
            id: self.id.clone(),
            value: self.value,
        };
        self.name.clear();
        self.address.clear();
        self.id.clear();
        inheritor
    }
}


#[derive(Debug, Clone)]
struct NewGuardian{
    guardian_number: u8,
    name: String,
    id: String,
}
impl NewGuardian{
    fn new(guardian_number: u8) -> Self{
        NewGuardian{
            guardian_number,
            name: String::from(""),
            id: String::from(""),
        }
    }

    fn view(&self, i: usize) -> Element<GuardianMessage>{
        return column![
            text(format!("Guardian: {}", i)).size(50),
            text_input("Name:", &self.name, GuardianMessage::SetName),
            text_input("UserID:", &self.id, GuardianMessage::SetID),
        ]
        .max_width(800)
        .spacing(10)
        .padding(10)
        .align_items(Alignment::Center)
        .into();
    }

    fn update(&mut self, message: GuardianMessage){
        match message{
            GuardianMessage::SetName(x) => self.name = x,
            GuardianMessage::SetID(x) => self.id = x,
        }
    }

    fn info(&mut self) -> Guardian{
        let guardian = Guardian{
            name: self.name.clone(),
            id: self.id.clone(),
        };
        self.name.clear();
        self.id.clear();
        guardian
    }
}


#[derive(Debug, Clone)]
struct SetLocktime{
    value: u32,
    unit: TimeUnit,
}
impl SetLocktime{
    fn new() -> Self{
        SetLocktime {
            value: 0,
            unit: TimeUnit::blocks,
        }
    }
    
    fn view(&self) -> Element<PageMessage>{
        return column![
            text(format!("Set Will Redemption Period")).size(50),
            text(format!("(Duration)")).size(40),
            column![text(format!("*Note: Important*")).size(20).style(theme::Text::Color(iced::Color::from_rgb8(255, 0, 0)))].width(Length::Fill).align_items(Alignment::Start),
            text(format!("If you lose access to your wallet for longer than set duration, your coins may move to your inheritor's wallets. Your guardian wallets may also be used to help you recover your coins if you have temporarily lose access to your main wallet within the set duration.")).size(20),
            vertical_space(Length::Units(60)),
            number_input_2(self.value, PageMessage::SetLocktime),
            pick_list(&TimeUnit::ALL[..], Some(self.unit), PageMessage::SetTimeUnit),
        ]
        .max_width(800)
        .spacing(10)
        .padding(10)
        .align_items(Alignment::Center)
        .into();
    }

    fn set_value(&mut self, timelock_value: u32){
        self.value = timelock_value
    }

    fn set_unit(&mut self, timelock_unit: TimeUnit){
        self.unit = timelock_unit
    }

    fn get_locktime_blocks(&self) -> u32{
        match self.unit{
            TimeUnit::blocks => self.value,
            TimeUnit::days => self.value * 144,
            TimeUnit::weeks => self.value * 1008,
            TimeUnit::months => self.value * 4320,
            TimeUnit::years => self.value * 52560,
        }
    }

}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeUnit{
    blocks,
    days,
    weeks,
    months,
    years,
}
impl TimeUnit{
    const ALL: [TimeUnit;5] = [
        TimeUnit::blocks,
        TimeUnit::days,
        TimeUnit::weeks,
        TimeUnit::months,
        TimeUnit::years,
    ];
}
impl Default for TimeUnit {
    fn default() -> TimeUnit {
        TimeUnit::blocks
    }
}
impl std::fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TimeUnit::blocks => "Blocks",
                TimeUnit::days => "Days",
                TimeUnit::weeks => "Weeks",
                TimeUnit::months => "Months",
                TimeUnit::years => "Years",
            }
        )
    }
}


mod numeric_input {
    use iced::alignment::{self, Alignment};
    use iced::widget::{self, button, row, column, text, text_input};
    use iced::{Element, Length};
    use iced_lazy::{self, Component};

    pub struct NumericInput<Message> {
        value: Option<u8>,
        on_change: Box<dyn Fn(Option<u8>) -> Message>,
    }

    pub fn numeric_input<Message>(
        value: Option<u8>,
        on_change: impl Fn(Option<u8>) -> Message + 'static,
    ) -> NumericInput<Message> {
        NumericInput::new(value, on_change)
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        InputChanged(String),
        IncrementPressed,
        DecrementPressed,
    }

    impl<Message> NumericInput<Message> {
        pub fn new(
            value: Option<u8>,
            on_change: impl Fn(Option<u8>) -> Message + 'static,
        ) -> Self {
            Self {
                value,
                on_change: Box::new(on_change),
            }
        }
    }

    impl<Message, Renderer> Component<Message, Renderer> for NumericInput<Message>
    where
        Renderer: iced_native::text::Renderer + 'static,
        Renderer::Theme: widget::button::StyleSheet
            + widget::text_input::StyleSheet
            + widget::text::StyleSheet,
    {
        type State = ();
        type Event = Event;

        fn update(
            &mut self,
            _state: &mut Self::State,
            event: Event,
        ) -> Option<Message> {
            match event {
                Event::IncrementPressed => Some((self.on_change)(Some(
                    self.value.unwrap_or_default().saturating_add(1),
                ))),
                Event::DecrementPressed => 
                    if self.value > Some(1){
                        Some((self.on_change)(Some(
                            self.value.unwrap_or_default().saturating_sub(1),
                        )))
                    }else{
                        Some((self.on_change)(Some(1)))
                    }
                Event::InputChanged(value) => {
                    if value.is_empty() {
                        Some((self.on_change)(None))
                    } else {
                        value
                            .parse()
                            .ok()
                            .map(Some)
                            .map(self.on_change.as_ref())
                    }
                }
            }
        }

        fn view(&self, _state: &Self::State) -> Element<Event, Renderer> {
            let button = |label, on_press| {
                button(
                    text(label)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center),
                )
                .width(Length::Units(50))
                .on_press(on_press)
            };

            column![
                button("+", Event::IncrementPressed),
                text_input(
                    "Type a number",
                    self.value
                        .as_ref()
                        .map(u8::to_string)
                        .as_deref()
                        .unwrap_or(""),
                    Event::InputChanged,
                )
                .padding(10),
                button("-", Event::DecrementPressed),
            ]
            .align_items(Alignment::Fill)
            .spacing(10)
            .into()
        }
    }

    impl<'a, Message, Renderer> From<NumericInput<Message>>
        for Element<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: 'static + iced_native::text::Renderer,
        Renderer::Theme: widget::button::StyleSheet
            + widget::text_input::StyleSheet
            + widget::text::StyleSheet,
    {
        fn from(numeric_input: NumericInput<Message>) -> Self {
            iced_lazy::component(numeric_input)
        }
    }
}

mod write_file{
use std::fs::File;
use std::io::prelude::*;

    pub fn write_file(data: String){
        let mut file = File::create("nInheritors.txt").expect("cannot create file");
        file.write_all(data.as_bytes()).expect("cannot write to file");
    }
}

mod number_input {
    use iced::alignment::{self, Alignment};
    use iced::widget::{self, button, row, column, text, text_input};
    use iced::{Element, Length};
    use iced_lazy::{self, Component};

    pub struct NumberInput<Message> {
        value: u64,
        on_change: Box<dyn Fn(u64) -> Message>,
    }

    pub fn number_input<Message>(
        value: u64,
        on_change: impl Fn(u64) -> Message + 'static,
    ) -> NumberInput<Message> {
        NumberInput::new(value, on_change)
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        InputChanged(String),
        IncrementPressed,
        DecrementPressed,
    }

    impl<Message> NumberInput<Message> {
        pub fn new(
            value: u64,
            on_change: impl Fn(u64) -> Message + 'static,
        ) -> Self {
            Self {
                value,
                on_change: Box::new(on_change),
            }
        }
    }

    impl<Message, Renderer> Component<Message, Renderer> for NumberInput<Message>
    where
        Renderer: iced_native::text::Renderer + 'static,
        Renderer::Theme: widget::button::StyleSheet
            + widget::text_input::StyleSheet
            + widget::text::StyleSheet,
    {
        type State = ();
        type Event = Event;

        fn update(
            &mut self,
            _state: &mut Self::State,
            event: Event,
        ) -> Option<Message> {
            match event {
                Event::IncrementPressed => Some((self.on_change)(
                    self.value.saturating_add(100000),
                )),
                Event::DecrementPressed => 
                        Some((self.on_change)(
                            self.value.saturating_sub(100000),
                        )),
                Event::InputChanged(value) => {
                    if value.is_empty() {
                        Some((self.on_change)(0))
                    } else {
                        value
                            .parse()
                            .ok()
                            .map(self.on_change.as_ref())
                    }
                }
            }
        }

        fn view(&self, _state: &Self::State) -> Element<Event, Renderer> {
            let button = |label, on_press| {
                button(
                    text(label)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center),
                )
                .width(Length::Units(50))
                .on_press(on_press)
            };

            row![
                button("-", Event::DecrementPressed),
                text_input(
                    "Type a number",
                    &self.value.to_string(),
                    Event::InputChanged,
                )
                .padding(10),
                button("+", Event::IncrementPressed),
            ]
            .align_items(Alignment::Fill)
            .spacing(10)
            .into()
        }
    }

    impl<'a, Message, Renderer> From<NumberInput<Message>>
        for Element<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: 'static + iced_native::text::Renderer,
        Renderer::Theme: widget::button::StyleSheet
            + widget::text_input::StyleSheet
            + widget::text::StyleSheet,
    {
        fn from(number_input: NumberInput<Message>) -> Self {
            iced_lazy::component(number_input)
        }
    }
}

mod number_input_1 {
    use iced::alignment::{self, Alignment};
    use iced::widget::{self, button, row, column, text, text_input};
    use iced::{Element, Length};
    use iced_lazy::{self, Component};

    pub struct NumberInput1<Message> {
        value: u64,
        on_change: Box<dyn Fn(u64) -> Message>,
    }

    pub fn number_input_1<Message>(
        value: u64,
        on_change: impl Fn(u64) -> Message + 'static,
    ) -> NumberInput1<Message> {
        NumberInput1::new(value, on_change)
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        InputChanged(String),
        IncrementPressed,
        DecrementPressed,
    }

    impl<Message> NumberInput1<Message> {
        pub fn new(
            value: u64,
            on_change: impl Fn(u64) -> Message + 'static,
        ) -> Self {
            Self {
                value,
                on_change: Box::new(on_change),
            }
        }
    }

    impl<Message, Renderer> Component<Message, Renderer> for NumberInput1<Message>
    where
        Renderer: iced_native::text::Renderer + 'static,
        Renderer::Theme: widget::button::StyleSheet
            + widget::text_input::StyleSheet
            + widget::text::StyleSheet,
    {
        type State = ();
        type Event = Event;

        fn update(
            &mut self,
            _state: &mut Self::State,
            event: Event,
        ) -> Option<Message> {
            match event {
                Event::IncrementPressed => Some((self.on_change)(
                    self.value.saturating_add(1),
                )),
                Event::DecrementPressed => 
                        Some((self.on_change)(
                            self.value.saturating_sub(1),
                        )),
                Event::InputChanged(value) => {
                    if value.is_empty() {
                        Some((self.on_change)(0))
                    } else {
                        value
                            .parse()
                            .ok()
                            .map(self.on_change.as_ref())
                    }
                }
            }
        }

        fn view(&self, _state: &Self::State) -> Element<Event, Renderer> {
            let button = |label, on_press| {
                button(
                    text(label)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center),
                )
                .width(Length::Units(50))
                .on_press(on_press)
            };

            row![
                button("-", Event::DecrementPressed),
                text_input(
                    "Type a number",
                    &self.value.to_string(),
                    Event::InputChanged,
                )
                .padding(10),
                button("+", Event::IncrementPressed),
            ]
            .align_items(Alignment::Fill)
            .spacing(10)
            .into()
        }
    }

    impl<'a, Message, Renderer> From<NumberInput1<Message>>
        for Element<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: 'static + iced_native::text::Renderer,
        Renderer::Theme: widget::button::StyleSheet
            + widget::text_input::StyleSheet
            + widget::text::StyleSheet,
    {
        fn from(number_input_1: NumberInput1<Message>) -> Self {
            iced_lazy::component(number_input_1)
        }
    }
}

mod number_input_2 {
    use iced::alignment::{self, Alignment};
    use iced::widget::{self, button, row, column, text, text_input};
    use iced::{Element, Length};
    use iced_lazy::{self, Component};

    pub struct NumberInput2<Message> {
        value: u32,
        on_change: Box<dyn Fn(u32) -> Message>,
    }

    pub fn number_input_2<Message>(
        value: u32,
        on_change: impl Fn(u32) -> Message + 'static,
    ) -> NumberInput2<Message> {
        NumberInput2::new(value, on_change)
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        InputChanged(String),
        IncrementPressed,
        DecrementPressed,
    }

    impl<Message> NumberInput2<Message> {
        pub fn new(
            value: u32,
            on_change: impl Fn(u32) -> Message + 'static,
        ) -> Self {
            Self {
                value,
                on_change: Box::new(on_change),
            }
        }
    }

    impl<Message, Renderer> Component<Message, Renderer> for NumberInput2<Message>
    where
        Renderer: iced_native::text::Renderer + 'static,
        Renderer::Theme: widget::button::StyleSheet
            + widget::text_input::StyleSheet
            + widget::text::StyleSheet,
    {
        type State = ();
        type Event = Event;

        fn update(
            &mut self,
            _state: &mut Self::State,
            event: Event,
        ) -> Option<Message> {
            match event {
                Event::IncrementPressed => Some((self.on_change)(
                    self.value.saturating_add(1),
                )),
                Event::DecrementPressed => 
                        Some((self.on_change)(
                            self.value.saturating_sub(1),
                        )),
                Event::InputChanged(value) => {
                    if value.is_empty() {
                        Some((self.on_change)(0))
                    } else {
                        value
                            .parse()
                            .ok()
                            .map(self.on_change.as_ref())
                    }
                }
            }
        }

        fn view(&self, _state: &Self::State) -> Element<Event, Renderer> {
            let button = |label, on_press| {
                button(
                    text(label)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center),
                )
                .width(Length::Units(50))
                .on_press(on_press)
            };

            row![
                button("-", Event::DecrementPressed),
                text_input(
                    "Type a number",
                    &self.value.to_string(),
                    Event::InputChanged,
                )
                .padding(10),
                button("+", Event::IncrementPressed),
            ]
            .align_items(Alignment::Fill)
            .spacing(10)
            .into()
        }
    }

    impl<'a, Message, Renderer> From<NumberInput2<Message>>
        for Element<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: 'static + iced_native::text::Renderer,
        Renderer::Theme: widget::button::StyleSheet
            + widget::text_input::StyleSheet
            + widget::text::StyleSheet,
    {
        fn from(number_input_1: NumberInput2<Message>) -> Self {
            iced_lazy::component(number_input_1)
        }
    }
}
