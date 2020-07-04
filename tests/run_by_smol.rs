use delay_timer::timer::timer::get_timestamp;
use futures::future;
use smol::{Task, Timer};
use std::{
    thread,
    time::{Duration, Instant},
};
#[test]
fn it_works() {
    //TODO: ONE THREAD , IS  serial.（一个线程的情况）
    //TODO: sleep let it block, i should try aother waiting method.
    //（thread::sleep，会阻塞执行的工作线程）

    // No need to `run()`, now we can just block on the main future.
    smol::run(async {
        let task = Task::spawn(async {
            thread::sleep(Duration::new(2, 0));
            println!("thread::sleep  会阻塞执行线程!{}", get_timestamp());
        });

        let tasg = Task::spawn(async {
            thread::sleep(Duration::new(2, 0));
            println!("2222222222222222222222222222222222:{}", get_timestamp());
        });

        future::join(task, tasg).await;
    })
}

#[test]
fn its_works() {
    //TODO: ONE THREAD , IS  serial.（同样，一个线程的情况）
    //TODO: sleep let it block, i should try aother waiting method.
    //（async fn sleep， 不会阻塞线程，因为他是一个状态机，poll了一下，没到时间就poll下一个）

    // No need to `run()`, now we can just block on the main future.
    smol::run(async {
        async fn sleep(dur: Duration) {
            Timer::after(dur).await;
        }
        let task = Task::spawn(async {
            sleep(Duration::from_secs(2)).await;
            println!("async fn sleep  并不会阻塞执行器的线程!{}", get_timestamp());
        });

        let tasg = Task::spawn(async {
            sleep(Duration::from_secs(2)).await;
            println!("2222222222222222222222222222222222:{}", get_timestamp());
        });

        future::join(task, tasg).await;
    })
}

#[test]
fn that_works() {
    //TODO: TWO THREAD , IS  divide equally .
    //两个线程去跑，一个阻塞了，另一个不阻塞的线程拿到另一个任务也去阻塞跑

    // A pending future is one that simply yields forever.
    thread::spawn(|| smol::run(future::pending::<()>()));

    // No need to `run()`, now we can just block on the main future.
    smol::run(async {
        let task = Task::spawn(async {
            thread::sleep(Duration::new(2, 0));
            println!("咱俩一块阻塞跑!{}", get_timestamp());
        });

        let tasg = Task::spawn(async {
            thread::sleep(Duration::new(2, 0));
            println!("2222222222222222222222222222222222:{}", get_timestamp());
        });

        future::join(task, tasg).await;
    });
}

#[test]
fn tasts_works() {
    use std::collections::HashMap;
    use std::future::Future;
    use std::pin::Pin;

    type BoxFn = Box<dyn Fn(i32) -> Pin<Box<dyn Future<Output = i32>>>>;

    fn insert<F: Fn(i32) -> R + 'static, R: Future<Output = i32> + 'static>(
        map: &mut HashMap<String, BoxFn>,
        name: String,
        f: F,
    ) {
        map.insert(name, Box::new(move |value| Box::pin(f(value))));
    }

    async fn a(n: i32) -> i32 {
        123
    }

    let mut map = HashMap::new();
    let a = a;
    let b = async { 123 };
    insert(&mut map, "a".to_string(), a);
}

#[test]
fn tasks_works() {
    use std::future::Future;
    use std::pin::Pin;

    type BoxFn = Box<dyn Fn(i32) -> Pin<Box<dyn Future<Output = i32>>>>;

    struct Task {
        pub task_id: u32,
        pub body: BoxFn,
        cylinder_line: u32,
        valid: bool,
    }

    async fn a(n: i32) -> i32 {
        let b = async { println!("i'bbbbbbbbbbbb") };
        b.await;
        println!("n:{}", n);
        n
    }
    use std::sync::Arc;
    let c = async { println!("123") };
    let d = async { println!("123") };
    let e = Arc::new(Box::new(d));
    let f = e.clone();

    fn create<F: Fn(i32) -> R + 'static, R: Future<Output = i32> + 'static>(f: F) -> BoxFn {
        Box::new(move |value| Box::pin(f(value)))
    }

    // let aa = a ;
    // let b = create(a);
    //单独放一个 future 放到task，会被耗尽
    //使用一个闭包，源源不断的生成 future
    //写两种task，一种是异步的，一种是同步的
    let bb: BoxFn = Box::new(move |value| Box::pin(a(value)));

    // let c = Box::new(move |value| Box::pin(a(value)));
    let task = Task {
        task_id: 1,
        body: bb,
        cylinder_line: 1,
        valid: true,
    };

    let t1 = (task.body)(1);
    let t2 = (task.body)(2);

    use smol::Task as SmolTask;
    smol::run(async {
        let task = SmolTask::local(t1);
        // let task1 = SmolTask::spawn(async{(task.body)(2)});

        let tasg = SmolTask::local(t2);

        future::join(task, tasg).await;
    });
}

#[test]
//TODO: 异步中调用同步，同步方法生成future ，丢到队列，然后去消费任务
//或许我可以写这种方法
//我的task，放一个同步指针，然后，同步中生成异步，让它自己跑去
//通过宏，让声明这个更简单
//方法2：我得task里面放一个闭包，但是生成的是dyn 的Future，只能在thread local中跑
//我可以实现两种不同的task
fn test_sync() {
    macro_rules! create_function {
        ($func_name:ident, $body:block) => {
            fn $func_name() {
                smol::Task::spawn(async { $body }).detach();

                println!("You called {:?}()", stringify!($func_name));
            }
        };
    }

    macro_rules! create_async_return_result {
        ($func_name:ident, $func:ident) => {
            create_function!($func_name, {
                let task = smol::Task::spawn(async { $func() });
                task.await;
            });
        };
    }

    create_function!(aaa, {
        println!("async0");
        let a = async { println!("[async]") };
        a.await;
    });

    create_async_return_result!(surft2, surft);

    struct Task {
        pub task_id: u32,
        pub body: fn(),
        cylinder_line: u32,
        valid: bool,
    }

    thread::spawn(|| smol::run(future::pending::<()>()));

    fn sync_task() {
        smol::Task::spawn(async {
            println!("123");
        })
        .detach();
    }

    let task = Task {
        task_id: 1,
        body: sync_task,
        cylinder_line: 1,
        valid: true,
    };
    let task2 = Task {
        task_id: 2,
        body: aaa,
        cylinder_line: 1,
        valid: true,
    };
    let task3 = Task {
        task_id: 3,
        body: aaa,
        cylinder_line: 1,
        valid: true,
    };

    async fn surft() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut res = surf::get("https://httpbin.org/get?id=555").await?;
        dbg!(res.body_string().await?);
        Ok(())
    }

    smol::run(async {
        let a = smol::Task::spawn(surft());
        a.unwrap().await;

        let a = smol::Task::spawn(async {
            let inner = smol::Task::spawn(surft());
            // let i = inner.unwrap();
            inner.unwrap().detach();
        });
    });
}

#[test]
fn test_child_process() {
    use std::fs::File;
    use std::io;
    use std::process::{ChildStdout, Command, Stdio};

    ///父进程拿着子进程的标准输出，copy到新的文件（会阻塞）
    // let child = Command::new("ping")
    // .args(&["-n", "10", "baidu.com"])
    // .stdout(Stdio::piped())
    // .spawn().unwrap();
    ///

    // println!("{:?}", child);
    // let mut f = File::create("./test1.log").unwrap();
    // io::copy(&mut child.stdout.unwrap(), &mut f).unwrap();

    /// 子进程挂上一个文件，做自己的标准输出,父进程跑自己的不阻塞
    let f = File::create("./foo.log").unwrap();
    // from_raw_fd is only considered unsafe if the file is used for mmap
    let child = Command::new("ping")
        .args(&["-n", "100", "baidu.com"])
        .stdout(f)
        .spawn()
        .unwrap();
    println!("{:?}, father bye bye.", child);

    //TODO:标准的闭包
    let a: Box<dyn Fn() + Send + Sync> = Box::new(|| {
        let f = File::create("./foo.log").unwrap();
        let child = Command::new("ping")
            .args(&["-n", "100", "baidu.com"])
            .stdout(f)
            .spawn()
            .unwrap();
        println!("{:?}, father bye bye.", child);
    });

    //child默认继承父亲的标准输出
    //重定向：https://stackoverflow.com/questions/42101920/how-to-redirect-child-process-output-to-stderr
    //https://www.e-learn.cn/topic/3203603
    //php a.php | sed *** >> a.txt  这是通过shell解析的命令，原生的程序不可以这么写
    //https://stackoverflow.com/questions/31666936/execute-a-shell-command
}