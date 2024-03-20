mod test_backtrace {
    use eyre::{eyre, Report, WrapErr};

    #[allow(unused_variables)]
    #[allow(dead_code)]
    enum FailFrame {
        None,
        Low,
        Med,
        High,
    }

    fn low(frame: FailFrame) -> Result<(), Report> {
        let e: Report = eyre!("This program's goodness is suspect!");
        if let FailFrame::Low = frame {
            Err::<(), Report>(e).wrap_err("The low-level code has failed!")
        } else {
            Ok(())
        }
    }

    fn med(frame: FailFrame) -> Result<(), Report> {
        let e: Report = eyre!("This program's goodness is suspect!");
        if let FailFrame::Med = frame {
            Err(e).wrap_err("The low-level code has failed!")
        } else {
            low(frame)
        }
    }
    fn high(frame: FailFrame) -> Result<(), Report> {
        let e: Report = eyre!("This program's goodness is suspect!");
        if let FailFrame::High = frame {
            Err(e).wrap_err("The low-level code has failed!")
        } else {
            med(frame)
        }
    }

    use std::panic;

    static BACKTRACE_SNIPPET_HIGH: &str = "
10: test_backtrace::test_backtrace::low
        at .\\tests\\test_backtrace.rs:14
11: test_backtrace::test_backtrace::med
        at .\\tests\\test_backtrace.rs:27
12: test_backtrace::test_backtrace::high
        at .\\tests\\test_backtrace.rs:35
13: test_backtrace::test_backtrace::test_backtrace
        ";

    use std::backtrace::Backtrace;
    use std::sync::{Arc, Mutex};

    /* This test does produce a backtrace for panic or error with the standard panic hook,
     * but I'm at a loss for how to capture the backtrace and compare it to a snippet.
     */
    #[cfg_attr(not(backtrace), ignore)]
    //    #[test]
    //   #[should_panic]
    fn test_backtrace_simple() {
        let report = high(FailFrame::Low).expect_err("Must be error");
        let handler: &eyre::DefaultHandler = report.handler().downcast_ref().unwrap();
        eprintln!("{:?}", handler);
        //        let backtrace: Backtrace = handler.backtrace.unwrap();
        // let
        /*
        let backtrace: Option<Backtrace> = handler.backtrace;
        assert!(backtrace.is_some());
        */
    }

    #[cfg_attr(not(backtrace), ignore)]
    //    #[test]
    fn test_backtrace() {
        /* FIXME: check that the backtrace actually happens here
         * It's not trivial to compare the *whole* output,
         * but we could somehow grep the output for 'stack_backtrace',
         * maybe check for this string... though including line numbers is problematic,
         * and the frames could change if core changes.
         *
         */

        let global_buffer = Arc::new(Mutex::new(String::new()));
        let old_hook = panic::take_hook();
        panic::set_hook({
            /* fixme: this panic hook is not working ;(
             */
            let global_buffer = global_buffer.clone();
            Box::new(move |info| {
                let mut global_buffer = global_buffer.lock().unwrap();

                if let Some(s) = info.payload().downcast_ref::<&str>() {
                    println!("PANIC: {}", *s);
                    *global_buffer = (*s).to_string()
                } else {
                    //                    panic!("help!");
                }
            })
        });

        panic::catch_unwind(|| {
            high(FailFrame::Low).unwrap(); //.unwrap_or(println!("test"));
        })
        .expect_err("Backtrace test did not panic.");
        let binding = global_buffer.lock().unwrap();
        let panic_output = binding.clone();
        panic::set_hook(old_hook);
        if !panic_output.contains(BACKTRACE_SNIPPET_HIGH) {
            println!("Backtrace test fail.");
            println!("Expected output to contain:");
            println!("{}", BACKTRACE_SNIPPET_HIGH);
            println!("Instead, outputted:");
            println!("{}", panic_output);
            panic!();
        }
    }
}
