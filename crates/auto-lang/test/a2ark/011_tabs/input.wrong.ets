import { Button } from '@kit.ArkUI';

@Preview
@Component
export struct TestTabs {
  @State currentIndex: number = 0
  private tabsController: TabsController = new TabsController()

    @Builder
    tabBarBuilder(title: string, targetIndex: number, selectedIcon?: Resource, unselectIcon?: Resource) {
      Column() {
        if (selectedIcon && unselectIcon) {
          Image(this.currentIndex === targetIndex ? selectedIcon : unselectIcon)
            .width(24)
            .height(24)
        }
        Text(title)
          .fontFamily('HarmonyHeiTi-Medium')
          .fontSize(10)
          .fontColor(this.currentIndex === targetIndex ? '#0A59F7' : 'rgba(0,0,0,0.60)')
          .textAlign(TextAlign.Center)
          .lineHeight(14)
          .fontWeight(500)
      }
      .width('100%')
      .height('100%')
      .justifyContent(FlexAlign.Center)
      .alignItems(HorizontalAlign.Center)
      .onClick(() => {
        this.currentIndex = targetIndex
        this.tabsController.changeIndex(targetIndex)
      })
    }

  build() {
    Column() {
          Text('Tab Navigation')
                .fontSize(18)
                .fontWeight(FontWeight.Bold)
              Tabs({ barPosition: BarPosition.End, controller: this.tabsController }) {
                TabContent() {
                  Text('Welcome to the home page')
                }
                .tabBar(this.tabBarBuilder('Home', 0))
                TabContent() {
                  Text('Settings page')
                }
                .tabBar(this.tabBarBuilder('Settings', 1))
              }
              .vertical(false)
              .scrollable(false)
              .backgroundColor('#F1F3F5')
        }
        .padding(16)
  }
}
