import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestTabs {
  build() {
    NavDestination() {
      Column() {
              Text('Tab Navigation')
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
    }
  }
}
