
// page.ets
import { Button } from '@kit.ArkUI';

enum Msg {
  Inc,
  Dec,
}

@Entry
@Component
struct Index {
  @State message: string = 'Hello World';
  @State count: number = 0;

  private dispatch(msg: Msg): void {
    switch (msg) {
    case Msg.Inc: {
      this.count = this.count + 1
      break;
    }
    case Msg.Dec: {
      this.count = this.count - 1
      break;
    }
  }
}

  build() {
    Column() {
      // 计数器显示区域
      Text(`Counter now: ${this.count}`)
        .fontSize(30)
        .margin(20)

      // 操作按钮区域
      Row() {
        Button('-')
          .onClick(() => {
            this.dispatch(Msg.Dec);
          })
          .width(100)
          .height(50)
          .margin(10)

        Button('+')
          .onClick(() => {
            this.dispatch(Msg.Inc);
          })
          .width(100)
          .height(50)
          .margin(10)
      }
    }
    .width('100%')
    .height('100%')
    .justifyContent(FlexAlign.Center)
  }
}
