use futures::FutureExt;
use gtk::glib::Sender;

use crate::{message::Message, window::{DISPLAYING_DIALOG, PIN_CODE, PASS_KEY, CONFIRMATION_AUTHORIZATION}};

async fn request_pin_code(request: bluer::agent::RequestPinCode, sender: Sender<Message>) -> bluer::agent::ReqResult<String> {
    println!("request pincode incoming");

    sender.send(Message::RequestPinCode(request)).expect("cannot send message");
    unsafe {
        DISPLAYING_DIALOG = true;
    }
    
    wait_for_dialog_exit().await;

    let final_pin_code = unsafe {
        PIN_CODE.clone()
    };
    println!("pin code is: {:?}", final_pin_code);
    if final_pin_code.is_empty() {
      	Err(bluer::agent::ReqError::Rejected)
    }
    else {
	    Ok(final_pin_code)
    }
}

async fn display_pin_code(request: bluer::agent::DisplayPinCode, sender: Sender<Message>) -> bluer::agent::ReqResult<()> {
    println!("display pincode incoming");
    
    sender.send(Message::DisplayPinCode(request)).expect("cannot send message");
    unsafe {
        DISPLAYING_DIALOG = true
    }

    wait_for_dialog_exit().await;

    println!("displaying pin code finished");
    Ok(())
}

async fn request_pass_key(request: bluer::agent::RequestPasskey, sender: Sender<Message>) -> bluer::agent::ReqResult<u32> {
    println!("request passkey incoming");

    sender.send(Message::RequestPassKey(request)).expect("cannot send message");
    unsafe {
        DISPLAYING_DIALOG = true;
    }

    wait_for_dialog_exit().await;

    let pass_key = unsafe {
        PASS_KEY
    };
    println!("pass key is: {}", pass_key);
    if pass_key == 0 {
    	Err(bluer::agent::ReqError::Rejected)
    }
    else {
    	Ok(pass_key)
    }
}   

async fn display_pass_key(request: bluer::agent::DisplayPasskey, sender: Sender<Message>) -> bluer::agent::ReqResult<()> {
    println!("display passkey incoming");
    
    sender.send(Message::DisplayPassKey(request)).expect("cannot send message");
    unsafe {
        DISPLAYING_DIALOG = true;
    }

    wait_for_dialog_exit().await;

    Ok(())
}

async fn request_confirmation(request: bluer::agent::RequestConfirmation, _: bluer::Session, _: bool, sender: Sender<Message>) -> bluer::agent::ReqResult<()> {
    println!("pairing confirmation incoming");
    
    sender.send(Message::RequestConfirmation(request)).expect("cannot send message");
    unsafe {
        DISPLAYING_DIALOG = true;
    }

    wait_for_dialog_exit().await;
    
    let confirmed = unsafe {
        CONFIRMATION_AUTHORIZATION
    };
    if confirmed {
        println!("allowed pairing with device");
        Ok(())
    }
    else {
        println!("rejected pairing with device");
        Err(bluer::agent::ReqError::Rejected)
    }
}

async fn request_authorization(request: bluer::agent::RequestAuthorization, _: bluer::Session, _: bool, sender: Sender<Message>) -> bluer::agent::ReqResult<()> {
    println!("pairing authorization incoming");

    sender.send(Message::RequestAuthorization(request)).expect("cannot send message");
    unsafe{
        DISPLAYING_DIALOG = true;
    }

    wait_for_dialog_exit().await;

    let confirmed = unsafe {
        CONFIRMATION_AUTHORIZATION
    };
    if confirmed {
        println!("allowed pairing with device");
        Ok(())
    }
    else {
        println!("rejected pairing with device");
        Err(bluer::agent::ReqError::Rejected)
    }

}

async fn authorize_service(request: bluer::agent::AuthorizeService, sender: Sender<Message>) -> bluer::agent::ReqResult<()> {
    println!("service authorization incoming");

    sender.send(Message::AuthorizeService(request)).expect("cannot send message");
    unsafe{
        DISPLAYING_DIALOG = true;
    }

    wait_for_dialog_exit().await;

    let confirmed = unsafe {
        CONFIRMATION_AUTHORIZATION
    };

    if confirmed {
        println!("allowed pairing with device");
        Ok(())
    }
    else {
        println!("rejected pairing with device");
        Err(bluer::agent::ReqError::Rejected)
    }
}

pub async fn register_agent(session: &bluer::Session, request_default: bool, set_trust: bool, sender_to_be_sent: Sender<Message>) -> bluer::Result<bluer::agent::AgentHandle> {
    let session1 = session.clone();
    let session2 = session.clone();

    // IDK if this is the best way, but its a way.
    let sender1 = sender_to_be_sent.clone();
    let sender2 = sender_to_be_sent.clone();
    let sender3 = sender_to_be_sent.clone();
    let sender4 = sender_to_be_sent.clone();
    let sender5 = sender_to_be_sent.clone();
    let sender6 = sender_to_be_sent.clone();
    let sender7 = sender_to_be_sent.clone();

    let agent = bluer::agent::Agent {
        request_default,
        request_pin_code: Some(Box::new(move |req| request_pin_code(req, sender1.clone()).boxed())),
        display_pin_code: Some(Box::new(move |req| display_pin_code(req, sender2.clone()).boxed())),
        request_passkey: Some(Box::new(move |req| request_pass_key(req, sender3.clone()).boxed())),
        display_passkey: Some(Box::new(move |req| display_pass_key(req, sender4.clone()).boxed())),
        request_confirmation: Some(Box::new(move |req| {
            request_confirmation(req, session1.clone(), set_trust, sender5.clone()).boxed()
        })),
        request_authorization: Some(Box::new(move |req| {
            request_authorization(req, session2.clone(), set_trust, sender6.clone()).boxed()
        })),
        authorize_service: Some(Box::new(move |req| authorize_service(req, sender7.clone()).boxed())),
        ..Default::default()
    };

    let handle = session.register_agent(agent).await.expect("unable to register agent, fuck-");
    
    Ok(handle)
}

async fn wait_for_dialog_exit() {
    unsafe {
        loop {
            if !DISPLAYING_DIALOG {
				// std::thread::sleep(std::time::Duration::from_secs(1));
                break;
            }
        }
    }
}
