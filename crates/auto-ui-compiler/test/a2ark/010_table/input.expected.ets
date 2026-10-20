import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestTable {
  build() {
        Column() {
          Text('Users Table')
          .fontSize(18)
          .fontWeight(FontWeight.Bold)
          Table() {
            Column() {
              Row() {
                Text() { }
                Name
                Text() { }
                Email
              }
            }
            Column() {
              Row() {
                Text() { }
                John
                Text() { }
                john@example.com
              }
            }
          }
        }
        .padding(16)
  }
}
