
enum TrafficLight {
    Red,
    Green,
    Yellow,
}

trait LightTime {
    fn time(&self) -> u8;
}

impl LightTime for TrafficLight {
    fn time(&self) -> u8 {
        match self {
            TrafficLight::Green => 60,
            TrafficLight::Red => 120,
            TrafficLight::Yellow => 3,
            _ => 0
        }
    }
}

fn total(list: &[u32])->Option<u32>{
    let mut  item= list.iter();
    item.try_fold(0u32,|acc,&item| acc.checked_add(item))
}

trait AreaType {
    fn area_types_print(&self);
}

struct LongBox {
    width:u64,
    long:u64,
}

struct SquareBox {
    width:u64,
    long:u64,
}

impl AreaType for LongBox {
    fn area_types_print(&self){
        println!("area is（长方形的面积是） {}",self.long*self.width);
    }
}
impl AreaType for SquareBox {
    fn area_types_print(&self){
        println!("area is（正方形的面积是） {}",self.long*self.width);
    }
}

fn area_print<T: AreaType>(item:T){
    item.area_types_print();
}

fn main() {
    println!("作业一 -> 为枚举交通信号灯实现一个 trait，trait里包含一个返回时间的方法，不同的灯持续的时间不同");
    let green = TrafficLight::Green;
    let red = TrafficLight::Red;
    let yellow = TrafficLight::Yellow;
    println!("yellow(黄色) time is {:?}",yellow.time());
    println!("red（红色） time is {:?}",red.time());
    println!("green（绿色） time is {:?}",green.time());
    println!("作业一 -> END");

    println!("作业二 -> 实现一个函数，为u32类型的整数集合求和，参数类型为 &[u32]，返回类型为Option，溢出时返回None");

    println!("作业二 -> 2.1 正常输出");
    let list = [1, 2, 3, 4, 5];
    let option = total(&list);
    match option {
        Some(c) => println!("list的总和为 {}", c),
        None => {
            println!("list溢出")
        }
    }

    println!("作业二 -> 2.2 溢出");
    let overflow_list = [231231233, 231231233, 231231233, 755676754, 3123464655];
    let overflow_list_option = total(&overflow_list);
    match overflow_list_option {
        Some(c) => println!("list的总和为 {}", c),
        None => {
            println!("list溢出")
        }
    }
    println!("作业二 -> END");

    println!("作业三 -> 实现一个打印图形面积的函数，它接收一个可以计算面积的类型作为参数，比如圆形，三角形，正方形，需要用到泛型和泛型约束");
    let long_box = LongBox {width: 30, long: 20};
    let square_box = SquareBox {width: 20, long: 20};
    area_print(long_box);
    area_print(square_box);
    println!("作业三 -> END");
}
