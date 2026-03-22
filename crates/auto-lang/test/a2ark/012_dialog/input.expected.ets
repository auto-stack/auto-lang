import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestDialog {
  build() {
    Column() {
          Text('Dialog Demo')
                .fontSize(18)
                .fontWeight(FontWeight.Bold)
          AlertDialog() {
                  Button() {
                            Text('Open Dialog')
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
