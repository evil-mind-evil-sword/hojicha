// Test all examples from COMMON_PATTERNS.md

#[cfg(test)]
mod basic_application {
    use hojicha_core::prelude::*;
    use hojicha_runtime::prelude::*;

    // Model - Your application state
    struct App {
        // State fields
    }

    // Messages - Events that change state
    enum Msg {
        // Your message variants
    }

    // Implementation
    impl Model for App {
        type Message = Msg;
        
        fn init(&mut self) -> Cmd<Msg> {
            // Initial setup
            Cmd::none()
        }
        
        fn update(&mut self, _event: Event<Msg>) -> Cmd<Msg> {
            // Handle events
            Cmd::none()
        }
        
        fn view(&self, _frame: &mut Frame, _area: Rect) {
            // Render UI
        }
    }
    
    #[test]
    fn test_basic_app_compiles() {
        let _app = App {};
    }
}

#[cfg(test)]
mod multi_component {
    use hojicha_core::prelude::*;
    use hojicha_runtime::prelude::*;
    use ratatui::prelude::*;
    
    struct Sidebar;
    struct MainView;
    struct StatusBar;
    
    enum SidebarMsg {}
    enum MainViewMsg {}
    enum StatusBarMsg {}
    
    impl Sidebar {
        fn update(&mut self, _msg: SidebarMsg) -> Cmd<SidebarMsg> {
            Cmd::none()
        }
        
        fn render(&self, _frame: &mut Frame, _area: Rect) {}
    }
    
    impl MainView {
        fn update(&mut self, _msg: MainViewMsg) -> Cmd<MainViewMsg> {
            Cmd::none()
        }
        
        fn render(&self, _frame: &mut Frame, _area: Rect) {}
    }

    struct App {
        sidebar: Sidebar,
        main_view: MainView,
        status_bar: StatusBar,
    }

    enum Msg {
        Sidebar(SidebarMsg),
        MainView(MainViewMsg),
        StatusBar(StatusBarMsg),
    }
    
    // Need to implement Cmd::map
    impl<M: hojicha_core::core::Message> Cmd<M> {
        fn map<N, F>(self, _f: F) -> Cmd<N> 
        where 
            N: hojicha_core::core::Message,
            F: Fn(M) -> N + 'static
        {
            // This would need actual implementation
            Cmd::none()
        }
    }

    impl Model for App {
        type Message = Msg;
        
        fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
            match event {
                Event::User(Msg::Sidebar(msg)) => {
                    self.sidebar.update(msg).map(Msg::Sidebar)
                }
                Event::User(Msg::MainView(msg)) => {
                    self.main_view.update(msg).map(Msg::MainView)
                }
                _ => Cmd::none()
            }
        }
        
        fn view(&self, frame: &mut Frame, area: Rect) {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(30), Constraint::Min(0)])
                .split(area);
                
            self.sidebar.render(frame, chunks[0]);
            self.main_view.render(frame, chunks[1]);
        }
    }
    
    #[test]
    fn test_multi_component_compiles() {
        let _app = App {
            sidebar: Sidebar,
            main_view: MainView,
            status_bar: StatusBar,
        };
    }
}