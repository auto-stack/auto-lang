import { Button } from '@kit.ArkUI';

enum Msg {
  Increment,
  Decrement,
  Reset,
}

@Preview
@Component
export struct TestMsg {
    @State count: number = 0

    private dispatch(msg: Msg): void {
      switch (msg) {
        case Msg.Increment: {
          // TODO: implement handler
          break;
        }
        case Msg.Decrement: {
          // TODO: implement handler
          break;
        }
        case Msg.Reset: {
          // TODO: implement handler
          break;
        }
      }
    }

  build() {
        Column() {
          Text('Count: {count}')
          Button('Increment')
          .onClick(() => {
        // Unknown handler: Increment
      })
          Button('Decrement')
          .onClick(() => {
        // Unknown handler: Decrement
      })
          Button('Reset')
          .onClick(() => {
        // Unknown handler: Reset
      })
        }
  }
}
