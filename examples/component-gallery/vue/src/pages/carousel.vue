<!-- CarouselPage component - Auto-generated from Auto language -->
<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import Prism from 'prismjs'
import { Card } from '@/components/ui/card'
import { Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPrevious } from '@/components/ui/carousel'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'


const activeTab = ref<string>('preview')

const copiedCode = ref<string>('')
const showCarouselBasicCode = ref<boolean>(true)
const activeCarouselBasicTab = ref<string>('auto')
const showCarouselSizesCode = ref<boolean>(true)
const activeCarouselSizesTab = ref<string>('auto')
const showCarouselSpacingCode = ref<boolean>(true)
const activeCarouselSpacingTab = ref<string>('auto')
const showCarouselOrientationCode = ref<boolean>(true)
const activeCarouselOrientationTab = ref<string>('auto')
const showCarouselOptionsCode = ref<boolean>(true)
const activeCarouselOptionsTab = ref<string>('auto')

// Copy to clipboard function
async function copyCode(code: string, id: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(code)
    copiedCode.value = id
    setTimeout(() => {
      copiedCode.value = ''
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

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
const codeblock1Code = `npx shadcn-vue@latest add carousel`
watch(activeCarouselBasicTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeCarouselSizesTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeCarouselSpacingTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeCarouselOrientationTab, () => {
  nextTick(() => Prism.highlightAll())
})
watch(activeCarouselOptionsTab, () => {
  nextTick(() => Prism.highlightAll())
})
onMounted(() => {
  nextTick(() => Prism.highlightAll())
})


</script>

<template>
  <div class="flex flex-col">
    <div class="flex flex-col gap-4">
      <h1 class="text-4xl font-bold tracking-tight">Carousel</h1>
      A carousel with motion and gestures built in using Embla Carousel.
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Installation</h2>
      <div class="relative rounded-lg border overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 bg-muted border-b">
          <span class="text-xs text-muted-foreground font-medium">bash</span>
          <button
            @click="copyCode(codeblock1Code, 'codeblock1')"
            class="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
          >
            <svg v-if="copiedCode !== 'codeblock1'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
            {{ copiedCode === 'codeblock1' ? 'Copied!' : 'Copy' }}
          </button>
        </div>
        <pre class="p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))] overflow-x-auto"><code class="block font-mono !p-0 language-bash">{{ codeblock1Code }}</code></pre>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Basic</h2>
      <!-- Merged CarouselBasic Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
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
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showCarouselBasicCode = !showCarouselBasicCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showCarouselBasicCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showCarouselBasicCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeCarouselBasicTab = 'auto'"
                  :class="activeCarouselBasicTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeCarouselBasicTab = 'vue'"
                  :class="activeCarouselBasicTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeCarouselBasicTab === 'auto' ? carouselBasicAutoCode : carouselBasicVueCode, 'carousel-basic')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'carousel-basic'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'carousel-basic' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeCarouselBasicTab === 'auto' ? 'auto' : 'html')">{{ activeCarouselBasicTab === 'auto' ? carouselBasicAutoCode : carouselBasicVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Sizes</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Use the basis utility class on CarouselItem to set the size.</span>
      <!-- Merged CarouselSizes Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
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
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showCarouselSizesCode = !showCarouselSizesCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showCarouselSizesCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showCarouselSizesCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeCarouselSizesTab = 'auto'"
                  :class="activeCarouselSizesTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeCarouselSizesTab = 'vue'"
                  :class="activeCarouselSizesTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeCarouselSizesTab === 'auto' ? carouselSizesAutoCode : carouselSizesVueCode, 'carousel-sizes')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'carousel-sizes'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'carousel-sizes' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeCarouselSizesTab === 'auto' ? 'auto' : 'html')">{{ activeCarouselSizesTab === 'auto' ? carouselSizesAutoCode : carouselSizesVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Spacing</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Use pl-4 on CarouselItem and -ml-4 on CarouselContent for spacing.</span>
      <!-- Merged CarouselSpacing Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
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
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showCarouselSpacingCode = !showCarouselSpacingCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showCarouselSpacingCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showCarouselSpacingCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeCarouselSpacingTab = 'auto'"
                  :class="activeCarouselSpacingTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeCarouselSpacingTab = 'vue'"
                  :class="activeCarouselSpacingTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeCarouselSpacingTab === 'auto' ? carouselSpacingAutoCode : carouselSpacingVueCode, 'carousel-spacing')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'carousel-spacing'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'carousel-spacing' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeCarouselSpacingTab === 'auto' ? 'auto' : 'html')">{{ activeCarouselSpacingTab === 'auto' ? carouselSpacingAutoCode : carouselSpacingVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Orientation</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Use the orientation prop to set vertical or horizontal direction.</span>
      <!-- Merged CarouselOrientation Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
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
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showCarouselOrientationCode = !showCarouselOrientationCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showCarouselOrientationCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showCarouselOrientationCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeCarouselOrientationTab = 'auto'"
                  :class="activeCarouselOrientationTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeCarouselOrientationTab = 'vue'"
                  :class="activeCarouselOrientationTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeCarouselOrientationTab === 'auto' ? carouselOrientationAutoCode : carouselOrientationVueCode, 'carousel-orientation')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'carousel-orientation'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'carousel-orientation' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeCarouselOrientationTab === 'auto' ? 'auto' : 'html')">{{ activeCarouselOrientationTab === 'auto' ? carouselOrientationAutoCode : carouselOrientationVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Options</h2>
      <span class="text-muted-foreground leading-7 text-muted-foreground">Pass Embla options using the opts prop.</span>
      <!-- Merged CarouselOptions Component -->
      <div class="rounded-lg border overflow-hidden">
        <!-- Preview Area -->
        <div class="flex items-center justify-center p-4 min-h-[100px] bg-[hsl(var(--preview-bg))]">
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
        </div>
        <!-- Toggle Code Footer -->
        <div class="border-t">
          <button
            @click="showCarouselOptionsCode = !showCarouselOptionsCode"
            class="flex w-full items-center justify-between px-4 py-2 text-sm text-muted-foreground hover:bg-muted/50 transition-colors"
          >
            <span class="font-medium">Code</span>
            <svg
              :class="showCarouselOptionsCode ? 'rotate-180' : ''"
              xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="transition-transform duration-200"
            >
              <path d="m6 9 6 6 6-6"/>
            </svg>
          </button>
          <!-- Expandable Code Block -->
          <div v-if="showCarouselOptionsCode" class="border-t">
            <!-- Tabs (gray title bar) -->
            <div class="flex items-center justify-between bg-muted">
              <div class="flex">
                <button
                  @click="activeCarouselOptionsTab = 'auto'"
                  :class="activeCarouselOptionsTab === 'auto' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Auto
                </button>
                <button
                  @click="activeCarouselOptionsTab = 'vue'"
                  :class="activeCarouselOptionsTab === 'vue' ? 'bg-background text-foreground border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'"
                  class="px-4 py-2 text-xs font-medium transition-colors"
                >
                  Vue
                </button>
              </div>
              <button
                @click="copyCode(activeCarouselOptionsTab === 'auto' ? carouselOptionsAutoCode : carouselOptionsVueCode, 'carousel-options')"
                class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 mr-2 text-xs text-muted-foreground hover:bg-background hover:text-foreground transition-colors"
              >
                <svg v-if="copiedCode !== 'carousel-options'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
                {{ copiedCode === 'carousel-options' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <!-- Code content with syntax highlighting -->
            <pre class="overflow-x-auto p-4 text-sm bg-[hsl(var(--code-bg))] text-[hsl(var(--code-fg))]"><code :class="'block font-mono !p-0 language-' + (activeCarouselOptionsTab === 'auto' ? 'auto' : 'html')">{{ activeCarouselOptionsTab === 'auto' ? carouselOptionsAutoCode : carouselOptionsVueCode }}</code></pre>
          </div>
        </div>
      </div>
      <h2 class="text-2xl font-semibold tracking-tight mt-8">Properties</h2>
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
    </div>
  </div>

</template>

<style scoped>
/* Override Prism.js default styles */
pre[class*="language-"] {
  margin: 0;
}

/* Component styles */

</style>


