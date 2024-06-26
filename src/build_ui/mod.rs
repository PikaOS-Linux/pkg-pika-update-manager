use crate::apt_update_page;
use crate::apt_update_page::apt_update_page;
use crate::config::{APP_GITHUB, APP_ICON, APP_ID, VERSION};
use adw::prelude::*;
use adw::*;
use gtk::glib::{clone, MainContext};
use gtk::{License, Orientation};
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::thread;

pub fn build_ui(app: &adw::Application) {
    // setup glib
    gtk::glib::set_prgname(Some(t!("app_name").to_string()));
    glib::set_application_name(&t!("app_name").to_string());

    let internet_connected = Rc::new(RefCell::new(false));
    let (internet_loop_sender, internet_loop_receiver) = async_channel::unbounded();
    let internet_loop_sender = internet_loop_sender.clone();

    std::thread::spawn(move || loop {
        match Command::new("ping").arg("google.com").arg("-c 1").output() {
            Ok(t) if t.status.success() => internet_loop_sender
                .send_blocking(true)
                .expect("The channel needs to be open"),
            _ => internet_loop_sender
                .send_blocking(false)
                .expect("The channel needs to be open"),
        };
        thread::sleep(std::time::Duration::from_secs(5));
    });

    let window_banner = adw::Banner::builder().revealed(false).build();

    let internet_connected_status = internet_connected.clone();

    let internet_loop_context = MainContext::default();
    // The main loop executes the asynchronous block
    internet_loop_context.spawn_local(clone!(@weak window_banner => async move {
        while let Ok(state) = internet_loop_receiver.recv().await {
            let banner_text = t!("banner_text_no_internet").to_string();
            if state == true {
                *internet_connected_status.borrow_mut()=true;
                if window_banner.title() == banner_text {
                    window_banner.set_revealed(false)
                }
            } else {
                *internet_connected_status.borrow_mut()=false;
                if window_banner.title() != t!("banner_text_url_error").to_string() {
                window_banner.set_title(&banner_text);
                window_banner.set_revealed(true)
                    }
            }
        }
    }));

    let window_headerbar = adw::HeaderBar::builder()
        .title_widget(
            &adw::WindowTitle::builder()
                .title(t!("application_name"))
                .build(),
        )
        .build();

    let window_toolbar = adw::ToolbarView::builder().build();

    window_toolbar.add_top_bar(&window_headerbar);
    window_toolbar.add_top_bar(&window_banner);

    // create the main Application window
    let window = adw::ApplicationWindow::builder()
        // The text on the titlebar
        .title(t!("app_name"))
        // link it to the application "app"
        .application(app)
        // Add the box called "window_box" to it
        // Application icon
        .icon_name(APP_ICON)
        // Minimum Size/Default
        .width_request(700)
        .height_request(500)
        .content(&window_toolbar)
        // Startup
        .startup_id(APP_ID)
        // build the window
        .build();

    let credits_button = gtk::Button::builder()
        .icon_name("dialog-information-symbolic")
        .build();

    let credits_window = adw::AboutWindow::builder()
        .application_icon(APP_ICON)
        .application_name(t!("application_name"))
        .transient_for(&window)
        .version(VERSION)
        .hide_on_close(true)
        .developer_name(t!("developer_name"))
        .license_type(License::Gpl20)
        .issue_url(APP_GITHUB.to_owned() + "/issues")
        .build();

    window_headerbar.pack_end(&credits_button);
    credits_button
        .connect_clicked(clone!(@weak credits_button => move |_| credits_window.present()));

    // show the window
    window.present();

    window_toolbar.set_content(Some(&apt_update_page::apt_update_page(window)));
}
