import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestTabs {
  build() {
    Column() {
          Text('Tab Navigation')
                .fontSize(18)
                .fontWeight(FontWeight.Bold)
          Tabs() {
                  Row() {
                            Text()
                            Text()
                          }
                  TabContent() {
                            Text('Welcome to the home page')
                          }
                }
        }
        .padding(16)
  }
}
