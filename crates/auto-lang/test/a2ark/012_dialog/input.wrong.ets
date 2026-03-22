import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestDialog {
  build() {
    NavDestination() {
      Column() {
              Text('Dialog Demo')
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
    }
  }
}
