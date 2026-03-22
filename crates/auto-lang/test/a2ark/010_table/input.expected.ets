import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestTable {
  build() {
    NavDestination() {
      Column() {
              Text('Users Table')
              Table() {
                        Column() {
                                    Row() {
                                                  Text()
                                                                Name
                                                  Text()
                                                                Email
                                                }
                                  }
                        Column() {
                                    Row() {
                                                  Text()
                                                                John
                                                  Text()
                                                                john@example.com
                                                }
                                  }
                      }
            }
    }
  }
}
