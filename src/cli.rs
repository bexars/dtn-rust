use log::*;
use shrust::{Shell, ShellIO};
use std::io::prelude::*;
use crate::bus::ModuleMsgEnum::*;
use crate::router::RouterModule::*;
use crate::bus::ModuleMsgEnum;
use crate::router::RouterModule;
use msg_bus::{MsgBusHandle, Message};



pub struct CliManager {
    bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>,
    //rx: Arc<Mutex<Receiver<Message<ModuleMsgEnum>>>>,

}

impl CliManager {
    pub async fn new( bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>) -> Self {
        // let rx = bus_handle.register(RouterModule::CLI);
        Self {
            // rx:  Arc::new(Mutex::new(rx)),
            bus_handle,
        }
    }


    pub fn start(self) { //  -> tokio::task::JoinHandle<()> {
        // bus_tx.send(ModuleMsgEnum::MsgBus(BusMessage::SetTx(tx.clone(), RouterModule::CLI))).await.unwrap();
        // let mut bus_handle = self.bus_handle.clone();
        debug!("In CliManager.start()");
        let handle = tokio::task::spawn_blocking(move || CliManager::start_shell(self.bus_handle.clone()));
        // handle
//        CliManager::start_shell(self.bus_handle.clone()).await;
    }
    

    pub fn start_shell( bus_handle: MsgBusHandle<RouterModule, ModuleMsgEnum>) {
        let mut shell = Shell::new(bus_handle);
        // let bus_handle = bus_handle.clone();
        shell.new_command_noargs("hello", "Say 'hello' to the world", |io, _| {
            writeln!(io, "Hello World !!!")?;
            Ok(())
        });
        shell.new_command_noargs("halt", "Stop all processing and shutdown", |io, bh|  {
            writeln!(io, "Halting.")?;
            // let mut bus_handle = bus_handle.clone();

            tokio::task::spawn({
                let mut bh = bh.clone();
                async move { bh.send(System, SystemMsg(crate::router::SystemMessage::ShutdownRequested)).await; }
            });
            // MsgBusHandle::blocking_send(&'static mut bus_handle, RouterModule::Routing, ModuleMsgEnum::ShutdownNow);
            Ok(())
        });
        shell.new_command("show", "Display data", 1, |io, bh, arg| {
            match arg[0] {
                "conf" => {
                    let mut bh = bh;
                    let res = futures::executor::block_on(crate::conf::get_conf(bh));
                    write!(io, "{}", res);

                    // let res = futures::executor::block_on(bh.rpc(Configuration, MsgConf(crate::conf::ConfMessage::GetConfigString)));

                    // if let Ok(conf) = res {
                    //     if let MsgConf(crate::conf::ConfMessage::DataConfigString(conf_str)) = conf {
                    //         write!(io, "{}", conf_str);
                    //     }
                    // };
                },
                _ => {},
            }
            Ok(())
        });
        debug!("About to start shell");
        shell.run_loop(&mut ShellIO::default());

        // shell.run_loop(&mut ShellIO::new(tokio::io::stdin(), tokio::io::stdout()));            
        
//        let shell = shell.as_ref().to_owned();
//        let handle = tokio::task::spawn_blocking(move || shell.run_loop(&mut ShellIO::default()));
        // tokio::join!(handle);
        // let mut rt = tokio::runtime::Runtime::new().unwrap();
        // let res = rt.block_on(async move {
        //     tokio::spawn(async {
        //         tokio::task::block_in_place(move || shell.run_loop(&mut ShellIO::default()));
        //     }).await
        // });
        debug!("After block_in_place");
    }
}