<script setup lang="ts">
import { Card } from '@/components/ui/card'
import { Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPrevious } from '@/components/ui/carousel'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import ComponentDocPage from '@/components/ComponentDocPage.vue'
import DemoSection from '@/components/DemoSection.vue'

const carouselBasicAutoCode = `carousel (class: "w-full max-w-xs") {
    carousel-content {
        carousel-item {
            card {
                card-content (class: "flex aspect-square items-center justify-center p-6") {
                    span (class: "text-4xl font-semibold", text: "1") {}
                }
            }
        }
        carousel-item {
            card {
                card-content (class: "flex aspect-square items-center justify-center p-6") {
                    span (text: "2", class: "text-4xl font-semibold") {}
                }
            }
        }
        carousel-item {
            card {
                card-content (class: "flex aspect-square items-center justify-center p-6") {
                    span (text: "3", class: "text-4xl font-semibold") {}
                }
            }
        }
    }
    carousel-previous {}
    carousel-next {}
}
`
const carouselBasicVueCode = `<Carousel class="w-full max-w-xs">
  <CarouselContent>
    <CarouselItem>
      <Card>
        <div class="flex aspect-square items-center justify-center p-6">
          <span class="text-4xl font-semibold">1</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem>
      <Card>
        <div class="flex aspect-square items-center justify-center p-6">
          <span class="text-4xl font-semibold">2</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem>
      <Card>
        <div class="flex aspect-square items-center justify-center p-6">
          <span class="text-4xl font-semibold">3</span>
        </div>
      </Card>
    </CarouselItem>
  </CarouselContent>
  <CarouselPrevious />
  <CarouselNext />
</Carousel>
`
const carouselSizesAutoCode = `carousel (class: "w-full max-w-sm") {
    carousel-content {
        carousel-item (class: "basis-1/2 md:basis-1/3") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (class: "text-3xl font-semibold", text: "1") {}
                }
            }
        }
        carousel-item (class: "basis-1/2 md:basis-1/3") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (class: "text-3xl font-semibold", text: "2") {}
                }
            }
        }
        carousel-item (class: "basis-1/2 md:basis-1/3") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (text: "3", class: "text-3xl font-semibold") {}
                }
            }
        }
        carousel-item (class: "basis-1/2 md:basis-1/3") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (text: "4", class: "text-3xl font-semibold") {}
                }
            }
        }
        carousel-item (class: "basis-1/2 md:basis-1/3") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (text: "5", class: "text-3xl font-semibold") {}
                }
            }
        }
        carousel-item (class: "basis-1/2 md:basis-1/3") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (text: "6", class: "text-3xl font-semibold") {}
                }
            }
        }
    }
    carousel-previous {}
    carousel-next {}
}
`
const carouselSizesVueCode = `<Carousel class="w-full max-w-sm">
  <CarouselContent>
    <CarouselItem class="basis-1/2 md:basis-1/3">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">1</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="basis-1/2 md:basis-1/3">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">2</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="basis-1/2 md:basis-1/3">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">3</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="basis-1/2 md:basis-1/3">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">4</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="basis-1/2 md:basis-1/3">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">5</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="basis-1/2 md:basis-1/3">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">6</span>
        </div>
      </Card>
    </CarouselItem>
  </CarouselContent>
  <CarouselPrevious />
  <CarouselNext />
</Carousel>
`
const carouselSpacingAutoCode = `carousel (class: "w-full max-w-sm") {
    carousel-content (class: "-ml-4") {
        carousel-item (class: "pl-4 basis-1/2") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (class: "text-3xl font-semibold", text: "1") {}
                }
            }
        }
        carousel-item (class: "pl-4 basis-1/2") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (class: "text-3xl font-semibold", text: "2") {}
                }
            }
        }
        carousel-item (class: "pl-4 basis-1/2") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (class: "text-3xl font-semibold", text: "3") {}
                }
            }
        }
        carousel-item (class: "pl-4 basis-1/2") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (text: "4", class: "text-3xl font-semibold") {}
                }
            }
        }
    }
    carousel-previous {}
    carousel-next {}
}
`
const carouselSpacingVueCode = `<Carousel class="w-full max-w-sm">
  <CarouselContent class="-ml-4">
    <CarouselItem class="pl-4 basis-1/2">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">1</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="pl-4 basis-1/2">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">2</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="pl-4 basis-1/2">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">3</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="pl-4 basis-1/2">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">4</span>
        </div>
      </Card>
    </CarouselItem>
  </CarouselContent>
  <CarouselPrevious />
  <CarouselNext />
</Carousel>
`
const carouselOrientationAutoCode = `div (class: "py-12 flex justify-center") {
    carousel (orientation: "vertical", class: "w-full max-w-xs h-[200px]") {
        carousel-content (class: "-mt-1 h-[200px]") {
            carousel-item (class: "pt-1 md:basis-1/2") {
                card {
                    card-content (class: "flex items-center justify-center p-6") {
                        span (class: "text-3xl font-semibold", text: "1") {}
                    }
                }
            }
            carousel-item (class: "pt-1 md:basis-1/2") {
                card {
                    card-content (class: "flex items-center justify-center p-6") {
                        span (text: "2", class: "text-3xl font-semibold") {}
                    }
                }
            }
            carousel-item (class: "pt-1 md:basis-1/2") {
                card {
                    card-content (class: "flex items-center justify-center p-6") {
                        span (text: "3", class: "text-3xl font-semibold") {}
                    }
                }
            }
            carousel-item (class: "pt-1 md:basis-1/2") {
                card {
                    card-content (class: "flex items-center justify-center p-6") {
                        span (text: "4", class: "text-3xl font-semibold") {}
                    }
                }
            }
        }
        carousel-previous {}
        carousel-next {}
    }
}
`
const carouselOrientationVueCode = `<div class="py-12 flex justify-center">
  <Carousel orientation="vertical" class="w-full max-w-xs h-[200px]">
    <CarouselContent class="-mt-1 h-[200px]">
      <CarouselItem class="pt-1 md:basis-1/2">
        <Card>
          <div class="flex items-center justify-center p-6">
            <span class="text-3xl font-semibold">1</span>
          </div>
        </Card>
      </CarouselItem>
      <CarouselItem class="pt-1 md:basis-1/2">
        <Card>
          <div class="flex items-center justify-center p-6">
            <span class="text-3xl font-semibold">2</span>
          </div>
        </Card>
      </CarouselItem>
      <CarouselItem class="pt-1 md:basis-1/2">
        <Card>
          <div class="flex items-center justify-center p-6">
            <span class="text-3xl font-semibold">3</span>
          </div>
        </Card>
      </CarouselItem>
      <CarouselItem class="pt-1 md:basis-1/2">
        <Card>
          <div class="flex items-center justify-center p-6">
            <span class="text-3xl font-semibold">4</span>
          </div>
        </Card>
      </CarouselItem>
    </CarouselContent>
    <CarouselPrevious />
    <CarouselNext />
  </Carousel>
</div>
`
const carouselOptionsAutoCode = `carousel (align: "start", loop: "true", class: "w-full max-w-sm") {
    carousel-content {
        carousel-item (class: "basis-1/3") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (class: "text-3xl font-semibold", text: "1") {}
                }
            }
        }
        carousel-item (class: "basis-1/3") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (class: "text-3xl font-semibold", text: "2") {}
                }
            }
        }
        carousel-item (class: "basis-1/3") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (text: "3", class: "text-3xl font-semibold") {}
                }
            }
        }
        carousel-item (class: "basis-1/3") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (text: "4", class: "text-3xl font-semibold") {}
                }
            }
        }
        carousel-item (class: "basis-1/3") {
            card {
                card-content (class: "flex items-center justify-center p-6") {
                    span (class: "text-3xl font-semibold", text: "5") {}
                }
            }
        }
    }
    carousel-previous {}
    carousel-next {}
}
`
const carouselOptionsVueCode = `<Carousel :opts="{ align: 'start', loop: true }" class="w-full max-w-sm">
  <CarouselContent>
    <CarouselItem class="basis-1/3">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">1</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="basis-1/3">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">2</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="basis-1/3">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">3</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="basis-1/3">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">4</span>
        </div>
      </Card>
    </CarouselItem>
    <CarouselItem class="basis-1/3">
      <Card>
        <div class="flex items-center justify-center p-6">
          <span class="text-3xl font-semibold">5</span>
        </div>
      </Card>
    </CarouselItem>
  </CarouselContent>
  <CarouselPrevious />
  <CarouselNext />
</Carousel>
`
</script>

<template>
  <ComponentDocPage title="Carousel" description="A carousel with motion and gestures built in using Embla Carousel." installCommand="npx shadcn-vue@latest add carousel">
    <DemoSection title="Basic" id="carousel-basic" :autoCode="carouselBasicAutoCode" :vueCode="carouselBasicVueCode">
      <template #preview>
        <Carousel class="w-full max-w-xs">
              <CarouselContent>
                <CarouselItem>
                  <Card>
                    <div class="flex aspect-square items-center justify-center p-6">
                      <span class="text-4xl font-semibold">1</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem>
                  <Card>
                    <div class="flex aspect-square items-center justify-center p-6">
                      <span class="text-4xl font-semibold">2</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem>
                  <Card>
                    <div class="flex aspect-square items-center justify-center p-6">
                      <span class="text-4xl font-semibold">3</span>
                    </div>
                  </Card>
                </CarouselItem>
              </CarouselContent>
              <CarouselPrevious />
              <CarouselNext />
            </Carousel>
      </template>
    </DemoSection>
    <DemoSection title="Sizes" id="carousel-sizes" :autoCode="carouselSizesAutoCode" :vueCode="carouselSizesVueCode">
      <template #preview>
        <Carousel class="w-full max-w-sm">
              <CarouselContent>
                <CarouselItem class="basis-1/2 md:basis-1/3">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">1</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="basis-1/2 md:basis-1/3">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">2</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="basis-1/2 md:basis-1/3">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">3</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="basis-1/2 md:basis-1/3">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">4</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="basis-1/2 md:basis-1/3">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">5</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="basis-1/2 md:basis-1/3">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">6</span>
                    </div>
                  </Card>
                </CarouselItem>
              </CarouselContent>
              <CarouselPrevious />
              <CarouselNext />
            </Carousel>
      </template>
    </DemoSection>
    <DemoSection title="Spacing" id="carousel-spacing" :autoCode="carouselSpacingAutoCode" :vueCode="carouselSpacingVueCode">
      <template #preview>
        <Carousel class="w-full max-w-sm">
              <CarouselContent class="-ml-4">
                <CarouselItem class="pl-4 basis-1/2">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">1</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="pl-4 basis-1/2">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">2</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="pl-4 basis-1/2">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">3</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="pl-4 basis-1/2">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">4</span>
                    </div>
                  </Card>
                </CarouselItem>
              </CarouselContent>
              <CarouselPrevious />
              <CarouselNext />
            </Carousel>
      </template>
    </DemoSection>
    <DemoSection title="Orientation" id="carousel-orientation" :autoCode="carouselOrientationAutoCode" :vueCode="carouselOrientationVueCode">
      <template #preview>
        <div class="py-12 flex justify-center">
              <Carousel orientation="vertical" class="w-full max-w-xs h-[200px]">
                <CarouselContent class="-mt-1 h-[200px]">
                  <CarouselItem class="pt-1 md:basis-1/2">
                    <Card>
                      <div class="flex items-center justify-center p-6">
                        <span class="text-3xl font-semibold">1</span>
                      </div>
                    </Card>
                  </CarouselItem>
                  <CarouselItem class="pt-1 md:basis-1/2">
                    <Card>
                      <div class="flex items-center justify-center p-6">
                        <span class="text-3xl font-semibold">2</span>
                      </div>
                    </Card>
                  </CarouselItem>
                  <CarouselItem class="pt-1 md:basis-1/2">
                    <Card>
                      <div class="flex items-center justify-center p-6">
                        <span class="text-3xl font-semibold">3</span>
                      </div>
                    </Card>
                  </CarouselItem>
                  <CarouselItem class="pt-1 md:basis-1/2">
                    <Card>
                      <div class="flex items-center justify-center p-6">
                        <span class="text-3xl font-semibold">4</span>
                      </div>
                    </Card>
                  </CarouselItem>
                </CarouselContent>
                <CarouselPrevious />
                <CarouselNext />
              </Carousel>
            </div>
      </template>
    </DemoSection>
    <DemoSection title="Options" id="carousel-options" :autoCode="carouselOptionsAutoCode" :vueCode="carouselOptionsVueCode">
      <template #preview>
        <Carousel :opts="{ align: 'start', loop: true }" class="w-full max-w-sm">
              <CarouselContent>
                <CarouselItem class="basis-1/3">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">1</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="basis-1/3">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">2</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="basis-1/3">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">3</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="basis-1/3">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">4</span>
                    </div>
                  </Card>
                </CarouselItem>
                <CarouselItem class="basis-1/3">
                  <Card>
                    <div class="flex items-center justify-center p-6">
                      <span class="text-3xl font-semibold">5</span>
                    </div>
                  </Card>
                </CarouselItem>
              </CarouselContent>
              <CarouselPrevious />
              <CarouselNext />
            </Carousel>
      </template>
    </DemoSection>
    <template #properties>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Property</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Default</TableHead>
            <TableHead>Values</TableHead>
            <TableHead>Description</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow>
            <TableCell>opts</TableCell>
            <TableCell>object</TableCell>
            <TableCell>{}</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Embla carousel options</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>orientation</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"horizontal"</TableCell>
            <TableCell>"horizontal", "vertical"</TableCell>
            <TableCell>Carousel direction</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>align</TableCell>
            <TableCell>string</TableCell>
            <TableCell>"center"</TableCell>
            <TableCell>"start", "center", "end"</TableCell>
            <TableCell>Alignment of slides</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>loop</TableCell>
            <TableCell>boolean</TableCell>
            <TableCell>false</TableCell>
            <TableCell>true, false</TableCell>
            <TableCell>Enable infinite loop</TableCell>
          </TableRow>
          <TableRow>
            <TableCell>class</TableCell>
            <TableCell>string</TableCell>
            <TableCell>-</TableCell>
            <TableCell>-</TableCell>
            <TableCell>Additional CSS classes</TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </template>
  </ComponentDocPage>
</template>

<style scoped>
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>
