<script setup lang="ts">
import { ref, reactive, computed } from 'vue';

type Step = 1 | 2 | 3;
type Category = 'gents' | 'ladies' | 'all';
type LocationStatus = 'idle' | 'requesting' | 'success' | 'denied' | 'error';

interface Barber {
  id: string;
  name: string;
  distance: string;
  rating: string;
  active?: boolean;
  images: { src: string; alt: string }[];
}

interface Props {
  accessToken: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  (e: 'success'): void;
}>();

// --- Domain & Static Data ---
const styleTags: Record<Category, string[]> = {
  gents: [
    'SkinFade',
    'BeardSculpt',
    'TexturedCrop',
    'ClassicTaper',
    'BuzzCut',
    'CurlyCut',
    'ModernMullet',
    'HotTowelService',
    'Pompadour',
    'LineUp',
  ],
  ladies: [
    'SilkPress',
    'BobCut',
    'KnotlessBraids',
    'PixieCut',
    'CurlyLayers',
    'Balayage',
    'LocStyles',
    'Blowout',
    'WolfCut',
  ],
  all: [
    'NaturalCurls',
    'Undercut',
    'ProtectiveStyles',
    'ColorDesign',
    'LongLayers',
    'CreativeFade',
    'ShagCut',
    'BraidedStyles',
  ],
};

const barbers: Barber[] = [
  {
    id: 'karim-cutz',
    name: 'Karim Cutz',
    distance: '1.2 km away',
    rating: '4.9',
    active: true,
    images: [
      {
        src: 'https://images.unsplash.com/photo-1578390432942-d323db577792?auto=format&fit=crop&w=360&h=360&q=80',
        alt: "Close-up portrait showing a finished haircut",
      },
      {
        src: 'https://images.unsplash.com/photo-1672642150228-3fcd5826ec26?auto=format&fit=crop&w=360&h=360&q=80',
        alt: "Barber shaping a client's haircut",
      },
      {
        src: 'https://images.unsplash.com/photo-1582771498000-8ad44e6c84db?auto=format&fit=crop&w=360&h=360&q=80',
        alt: 'Haircut in progress at a barbershop',
      },
    ],
  },
  {
    id: 'yassine-studio',
    name: 'Yassine Studio',
    distance: '2.5 km away',
    rating: '4.8',
    images: [
      {
        src: 'https://images.unsplash.com/photo-1640301133543-41fe25ad6450?auto=format&fit=crop&w=360&h=360&q=80',
        alt: 'Barber refining a modern haircut',
      },
      {
        src: 'https://images.unsplash.com/photo-1514336937476-a5b961020a5c?auto=format&fit=crop&w=360&h=360&q=80',
        alt: 'Detailed haircut transformation',
      },
      {
        src: 'https://images.unsplash.com/photo-1669568921948-f0346e0ee73e?auto=format&fit=crop&w=360&h=360&q=80',
        alt: 'Client receiving a fresh haircut',
      },
    ],
  },
];

// --- Reactive State ---
const step = ref<Step>(1);
const activeCategory = ref<Category>('gents');
const selectedTags = reactive<Record<Category, string[]>>({
  gents: ['SkinFade', 'BeardSculpt'],
  ladies: [],
  all: [],
});
const city = ref('Algiers');
const radius = ref(10);
const locationStatus = ref<LocationStatus>('idle');
const followedIds = ref<string[]>([]);
const barberQuery = ref('');
const completed = ref(false);
const isLoading = ref(false);
const errorMessage = ref<string | null>(null);

// --- Computed Properties ---
const styleCount = computed(() =>
  Object.values(selectedTags).reduce((total, tags) => total + tags.length, 0)
);

const hasStyleSelection = computed(() =>
  Object.values(selectedTags).some((tags) => tags.length > 0)
);

const filteredBarbers = computed(() =>
  barbers.filter((b) =>
    b.name.toLowerCase().includes(barberQuery.value.trim().toLowerCase())
  )
);

const locationMessage = computed(() => {
  const map: Record<LocationStatus, string | null> = {
    idle: null,
    requesting: 'Waiting for location permission…',
    success: 'Current location detected. You can adjust the city below.',
    denied: 'Location permission was denied. Enter your city manually.',
    error: "We couldn't detect your location. Enter your city manually.",
  };
  return map[locationStatus.value];
});

// --- Action Handlers ---
function toggleTag(category: Category, tag: string) {
  const list = selectedTags[category];
  const index = list.indexOf(tag);
  if (index > -1) {
    list.splice(index, 1);
  } else {
    list.push(tag);
  }
}

function toggleFollow(barberId: string) {
  const index = followedIds.value.indexOf(barberId);
  if (index > -1) {
    followedIds.value.splice(index, 1);
  } else {
    followedIds.value.push(barberId);
  }
}

function requestLocation() {
  if (!navigator.geolocation) {
    locationStatus.value = 'error';
    return;
  }
  locationStatus.value = 'requesting';
  navigator.geolocation.getCurrentPosition(
    () => {
      city.value = 'Algiers';
      locationStatus.value = 'success';
    },
    (error) => {
      locationStatus.value =
        error.code === error.PERMISSION_DENIED ? 'denied' : 'error';
    },
    { enableHighAccuracy: false, timeout: 10000, maximumAge: 300000 }
  );
}

// --- Backend API Integration ---
async function finishAndLaunchFeed() {
  errorMessage.value = null;
  isLoading.value = true;

  // Flatten selected tags across categories for backend DTO
  const flattenedStyles = Array.from(
    new Set([...selectedTags.gents, ...selectedTags.ladies, ...selectedTags.all])
  );

  const payload = {
    city: city.value.trim(),
    search_radius_km: radius.value,
    selected_styles: flattenedStyles,
    followed_barber_ids: followedIds.value,
  };

  try {
    const response = await fetch('/api/v1/client/onboarding', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${props.accessToken}`,
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      const errJson = await response.json();
      throw new Error(errJson.error || errJson.message || 'Onboarding failed');
    }

    completed.value = true;
    emit('success');
  } catch (err: any) {
    errorMessage.value = err.message || 'An unexpected error occurred';
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <main className="min-h-screen bg-[#09090B] text-[#FAFAFA]">
    <div
      className="mx-auto flex min-h-screen w-full max-w-[480px] flex-col bg-[#09090B] shadow-[0_24px_80px_rgba(0,0,0,0.45)] sm:border-x sm:border-[#27272A]"
    >
      <!-- Error Notification Banner -->
      <div
        v-if="errorMessage"
        class="m-4 p-3 bg-red-950/80 border border-red-800 text-red-300 rounded-xl text-xs"
      >
        {{ errorMessage }}
      </div>

      <!-- COMPLETED STATE -->
      <div
        v-if="completed"
        className="flex min-h-[min(844px,100vh)] flex-col justify-center px-6 py-12 text-center"
      >
        <div
          className="mx-auto grid size-16 place-items-center rounded-full border border-[#F59E0B]/30 bg-[#F59E0B]/10 text-[#F59E0B]"
        >
          <svg className="size-8" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
            <path stroke-linecap="round" stroke-linejoin="round" d="m5 12 4 4L19 6" />
          </svg>
        </div>
        <p className="mt-6 text-xs font-semibold uppercase tracking-[0.16em] text-[#F59E0B]">
          Onboarding complete
        </p>
        <h1 className="mt-2 text-3xl font-bold tracking-[-0.03em] text-[#FAFAFA]">
          Your feed is ready
        </h1>
        <p className="mx-auto mt-3 max-w-sm text-sm leading-6 text-[#A1A1AA]">
          Your preferences are saved. Fresh local inspiration is ready to find.
        </p>

        <dl
          className="mt-9 grid grid-cols-3 divide-x divide-[#27272A] rounded-2xl border border-[#27272A] bg-[#18181B] px-2 py-5"
        >
          <div className="px-2">
            <dt className="text-[10px] uppercase tracking-wider text-[#71717A]">Styles</dt>
            <dd className="mt-1 text-lg font-bold text-[#FAFAFA]">{{ styleCount }}</dd>
          </div>
          <div className="px-2">
            <dt className="text-[10px] uppercase tracking-wider text-[#71717A]">Radius</dt>
            <dd className="mt-1 text-lg font-bold text-[#FAFAFA]">{{ radius }} km</dd>
          </div>
          <div className="px-2">
            <dt className="text-[10px] uppercase tracking-wider text-[#71717A]">Following</dt>
            <dd className="mt-1 text-lg font-bold text-[#FAFAFA]">{{ followedIds.length }}</dd>
          </div>
        </dl>

        <div className="mt-4 flex items-center justify-center gap-2 text-sm text-[#A1A1AA]">
          <svg className="size-4 text-[#F59E0B]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
            <path stroke-linecap="round" stroke-linejoin="round" d="M20 10c0 5-8 11-8 11S4 15 4 10a8 8 0 1 1 16 0Z" />
            <circle cx="12" cy="10" r="2.5" />
          </svg>
          {{ city || "Selected city" }}
        </div>
      </div>

      <!-- STEP 1: STYLE SELECTION -->
      <template v-else-if="step === 1">
        <div className="flex-1 px-4 pb-8 pt-5">
          <header>
            <div className="flex h-9 items-center justify-between">
              <span className="rounded-full border border-[#27272A] bg-[#18181B] px-3 py-1.5 text-xs font-medium text-[#71717A]">
                Step 1 of 3
              </span>
              <button
                className="rounded-lg px-2 py-2 text-sm font-semibold text-[#A1A1AA] transition hover:text-[#FAFAFA]"
                type="button"
                @click="step = 2"
              >
                Skip
              </button>
            </div>
            <div className="mt-7">
              <h1 className="text-2xl font-bold tracking-[-0.025em] text-[#FAFAFA]">What's your haircut vibe?</h1>
              <p className="mt-2 text-sm leading-6 text-[#A1A1AA]">Select style tags to customize your local feed.</p>
            </div>
          </header>

          <div className="mt-8 grid grid-cols-3 rounded-xl border border-[#27272A] bg-[#18181B] p-1">
            <button
              v-for="cat in (['gents', 'ladies', 'all'] as Category[])"
              :key="cat"
              type="button"
              :class="[
                'h-10 rounded-lg text-sm transition capitalize',
                activeCategory === cat
                  ? 'bg-[#F59E0B] font-semibold text-[#09090B]'
                  : 'font-medium text-[#A1A1AA] hover:text-[#FAFAFA]'
              ]"
              @click="activeCategory = cat"
            >
              {{ cat === 'all' ? 'All Styles' : cat }}
            </button>
          </div>

          <div className="mt-7">
            <p className="mb-4 text-xs font-semibold uppercase tracking-[0.14em] text-[#71717A]">
              Choose all that fit
            </p>
            <div className="flex flex-wrap gap-2.5">
              <button
                v-for="tag in styleTags[activeCategory]"
                :key="tag"
                type="button"
                :class="[
                  'rounded-full border px-4 py-2 text-sm font-medium transition',
                  selectedTags[activeCategory].includes(tag)
                    ? 'border-[#F59E0B] bg-[#F59E0B]/10 text-[#F59E0B]'
                    : 'border-[#27272A] bg-[#18181B] text-[#FAFAFA] hover:border-[#52525B]'
                ]"
                @click="toggleTag(activeCategory, tag)"
              >
                #{{ tag }}
              </button>
            </div>
          </div>
        </div>

        <div className="sticky bottom-0 mt-auto border-t border-[#27272A] bg-[#09090B]/95 px-4 pb-[max(20px,env(safe-area-inset-bottom))] pt-4 backdrop-blur-xl">
          <button
            className="h-12 w-full rounded-xl bg-[#F59E0B] text-sm font-bold text-[#09090B] transition hover:bg-[#D97706] disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="!hasStyleSelection"
            type="button"
            @click="step = 2"
          >
            Next
          </button>
        </div>
      </template>

      <!-- STEP 2: LOCATION & RADIUS -->
      <template v-else-if="step === 2">
        <div className="flex-1 px-4 pb-8 pt-5">
          <header>
            <div className="flex h-9 items-center justify-between">
              <span className="rounded-full border border-[#27272A] bg-[#18181B] px-3 py-1.5 text-xs font-medium text-[#71717A]">
                Step 2 of 3
              </span>
              <button
                className="rounded-lg px-2 py-2 text-sm font-semibold text-[#A1A1AA] transition hover:text-[#FAFAFA]"
                type="button"
                @click="step = 3"
              >
                Skip
              </button>
            </div>
            <div className="mt-7">
              <h1 className="text-2xl font-bold tracking-[-0.025em] text-[#FAFAFA]">Where do you get fresh?</h1>
              <p className="mt-2 text-sm leading-6 text-[#A1A1AA]">Discover top barbers and trending cuts near you.</p>
            </div>
          </header>

          <div className="mt-8 space-y-4">
            <button
              className="flex h-14 w-full items-center justify-between rounded-xl border border-[#27272A] bg-[#18181B] px-4 text-left transition hover:border-[#52525B] disabled:opacity-60"
              :disabled="locationStatus === 'requesting'"
              type="button"
              @click="requestLocation"
            >
              <span className="flex items-center gap-3">
                <svg
                  :class="['size-5 text-[#F59E0B]', locationStatus === 'requesting' ? 'animate-pulse' : '']"
                  fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" d="M20 10c0 5-8 11-8 11S4 15 4 10a8 8 0 1 1 16 0Z" />
                  <circle cx="12" cy="10" r="2.5" />
                </svg>
                <span className="text-sm font-medium text-[#FAFAFA]">
                  {{ locationStatus === 'requesting' ? 'Detecting Location…' : 'Use Current Location' }}
                </span>
              </span>
              <svg className="size-4 text-[#71717A]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
                <path stroke-linecap="round" stroke-linejoin="round" d="m9 18 6-6-6-6" />
              </svg>
            </button>

            <p
              v-if="locationMessage"
              :class="['px-1 text-xs leading-5', locationStatus === 'success' ? 'text-emerald-400' : 'text-[#A1A1AA]']"
            >
              {{ locationMessage }}
            </p>

            <label className="block">
              <span className="mb-2 block text-sm font-medium text-[#FAFAFA]">Selected City</span>
              <span className="relative block">
                <svg className="absolute left-4 top-1/2 size-4 -translate-y-1/2 text-[#71717A]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
                  <circle cx="11" cy="11" r="7" />
                  <path stroke-linecap="round" stroke-linejoin="round" d="m20 20-4-4" />
                </svg>
                <input
                  v-model="city"
                  className="h-14 w-full rounded-xl border border-[#27272A] bg-[#18181B] pl-11 pr-4 text-sm text-[#FAFAFA] outline-none transition placeholder:text-[#52525B] focus:border-[#F59E0B]"
                  placeholder="Search city or area"
                  type="text"
                />
              </span>
            </label>
          </div>

          <fieldset className="mt-8">
            <legend className="text-sm font-medium text-[#FAFAFA]">Search Radius</legend>
            <div className="mt-3 grid grid-cols-4 gap-2">
              <button
                v-for="option in [3, 5, 10, 25]"
                :key="option"
                type="button"
                :class="[
                  'h-10 rounded-full border text-xs transition',
                  radius === option
                    ? 'border-[#F59E0B] bg-[#F59E0B] font-bold text-[#09090B]'
                    : 'border-[#27272A] bg-[#18181B] font-medium text-[#A1A1AA] hover:border-[#52525B]'
                ]"
                @click="radius = option"
              >
                {{ option }} km
              </button>
            </div>
          </fieldset>

          <div className="mt-6 flex items-start gap-2.5 rounded-lg border border-[#78350F] bg-[#451A03] p-3">
            <svg className="mt-0.5 size-4 shrink-0 text-[#F59E0B]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
              <path stroke-linecap="round" stroke-linejoin="round" d="M20 10c0 5-8 11-8 11S4 15 4 10a8 8 0 1 1 16 0Z" />
              <circle cx="12" cy="10" r="2.5" />
            </svg>
            <p className="text-xs font-medium leading-5 text-[#F59E0B]">
              {{ city.trim() ? `14 Barbers active within ${radius} km of ${city.trim()}` : "Enter a city to discover active barbers nearby" }}
            </p>
          </div>
        </div>

        <div className="sticky bottom-0 mt-auto border-t border-[#27272A] bg-[#09090B]/95 px-4 pb-[max(20px,env(safe-area-inset-bottom))] pt-4 backdrop-blur-xl">
          <button
            className="h-12 w-full rounded-xl bg-[#F59E0B] text-sm font-bold text-[#09090B] transition hover:bg-[#D97706] disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="!city.trim()"
            type="button"
            @click="step = 3"
          >
            Set Location &amp; Continue
          </button>
        </div>
      </template>

      <!-- STEP 3: BARBER FOLLOW & SUBMIT -->
      <template v-else-if="step === 3">
        <div className="flex-1 px-4 pb-8 pt-5">
          <header>
            <div className="flex h-9 items-center justify-between">
              <span className="rounded-full border border-[#27272A] bg-[#18181B] px-3 py-1.5 text-xs font-medium text-[#71717A]">
                Step 3 of 3
              </span>
            </div>
            <div className="mt-7">
              <h1 className="text-2xl font-bold tracking-[-0.025em] text-[#FAFAFA]">Follow local trendsetters</h1>
              <p className="mt-2 text-sm leading-6 text-[#A1A1AA]">
                Seed your feed with fresh transformations in {{ city.trim() || "your area" }}.
              </p>
            </div>
          </header>

          <label className="relative mt-7 block">
            <span className="sr-only">Search for a barber by name</span>
            <svg className="absolute left-4 top-1/2 size-4 -translate-y-1/2 text-[#71717A]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
              <circle cx="11" cy="11" r="7" />
              <path stroke-linecap="round" stroke-linejoin="round" d="m20 20-4-4" />
            </svg>
            <input
              v-model="barberQuery"
              className="h-12 w-full rounded-xl border border-[#27272A] bg-[#18181B] pl-11 pr-4 text-sm text-[#FAFAFA] outline-none transition placeholder:text-[#71717A] focus:border-[#F59E0B]"
              placeholder="Search a preferred barber"
              type="search"
            />
          </label>

          <div className="mt-4 space-y-4">
            <template v-if="filteredBarbers.length > 0">
              <article
                v-for="b in filteredBarbers"
                :key="b.id"
                className="rounded-2xl border border-[#27272A] bg-[#18181B] p-4"
              >
                <div className="flex items-center gap-3">
                  <div className="relative grid size-11 shrink-0 place-items-center rounded-full bg-gradient-to-br from-[#F59E0B] to-[#92400E] text-sm font-extrabold text-[#09090B]">
                    {{ b.name.slice(0, 2).toUpperCase() }}
                    <span
                      v-if="b.active"
                      className="absolute bottom-0 right-0 size-3 rounded-full border-2 border-[#18181B] bg-emerald-400"
                    />
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <h2 className="truncate text-sm font-bold text-[#FAFAFA]">{{ b.name }}</h2>
                      <span className="shrink-0 text-xs font-semibold text-[#F59E0B]">★ {{ b.rating }}</span>
                    </div>
                    <p className="mt-0.5 text-xs text-[#71717A]">{{ b.distance }}</p>
                  </div>
                  <button
                    type="button"
                    :class="[
                      'h-8 shrink-0 rounded-lg border px-3 text-xs font-semibold transition',
                      followedIds.includes(b.id)
                        ? 'border-[#F59E0B] bg-[#F59E0B] text-[#09090B]'
                        : 'border-[#F59E0B] text-[#F59E0B] hover:bg-[#F59E0B] hover:text-[#09090B]'
                    ]"
                    @click="toggleFollow(b.id)"
                  >
                    {{ followedIds.includes(b.id) ? "Following" : "+ Follow" }}
                  </button>
                </div>
                <div className="mt-4 grid grid-cols-3 gap-2">
                  <img
                    v-for="img in b.images"
                    :key="img.src"
                    :alt="img.alt"
                    :src="img.src"
                    className="aspect-square w-full rounded-xl object-cover"
                    loading="lazy"
                  />
                </div>
              </article>
            </template>

            <div v-else className="rounded-xl border border-dashed border-[#27272A] bg-[#18181B] p-6 text-center">
              <h2 className="text-sm font-semibold text-[#FAFAFA]">No barbers match that name</h2>
              <p className="mt-2 text-xs text-[#A1A1AA]">Try another spelling or clear your search.</p>
              <button
                className="mt-4 text-xs font-semibold text-[#F59E0B] hover:underline"
                type="button"
                @click="barberQuery = ''"
              >
                Clear search
              </button>
            </div>
          </div>
        </div>

        <div className="sticky bottom-0 mt-auto border-t border-[#27272A] bg-[#09090B]/95 px-4 pb-[max(20px,env(safe-area-inset-bottom))] pt-4 backdrop-blur-xl">
          <button
            className="h-12 w-full rounded-xl bg-[#F59E0B] text-sm font-bold text-[#09090B] transition hover:bg-[#D97706] disabled:cursor-not-allowed disabled:opacity-40 flex justify-center items-center"
            :disabled="isLoading"
            type="button"
            @click="finishAndLaunchFeed"
          >
            <span v-if="isLoading" class="inline-block animate-spin rounded-full h-4 w-4 border-2 border-[#09090B] border-t-transparent mr-2" />
            Finish &amp; Launch Feed
          </button>
        </div>
      </template>
    </div>
  </main>
</template>