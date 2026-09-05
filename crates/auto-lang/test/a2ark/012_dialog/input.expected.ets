import { Button } from '@kit.ArkUI';

enum Msg {
  __dlg_toggle_1,
  __dlg_close_1,
}

@Preview
@Component
export struct TestDialog {
    @State __dlg_open_1: boolean = false

    private dispatch(msg: Msg): void {
      switch (msg) {
        case Msg.__dlg_toggle_1: {
          this.__dlg_open_1 = !this.__dlg_open_1;
          break;
        }
        case Msg.__dlg_close_1: {
          this.__dlg_open_1 = false;
          break;
        }
      }
    }

  build() {
        Column() {
          Text('Dialog Demo')
          .fontSize(18)
          .fontWeight(FontWeight.Bold)
          AlertDialog() {
            Button() {
              Text('Open Dialog')
              .onClick(() => {
        this.dispatch(Msg.__dlg_toggle_1)
      })
            }
            Column() {
              Column() {
                Text() {
                  Text('Confirm Action')
                }
              }
              Row() {
                Button('Cancel')
                Button('Confirm')
              }
            }
          }
        }
        .padding(16)
  }
}
