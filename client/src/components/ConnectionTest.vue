<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { transportApi, type SystemStatus } from '@/api/transport';

const restStatus = ref<SystemStatus | null>(null);
const restError = ref<string | null>(null);
const isWsConnected = ref<boolean>(false);
const wsMessages = ref<string[]>([]);
let socket: WebSocket | null = null;

async function pingRestServer() {
  try {
    restError.value = null;
    restStatus.value = await transportApi.checkHealth();
  } catch (err: any) {
    restError.value = err.message || 'Failed to ping backend';
  }
}

function sendWsPing() {
  if (socket && isWsConnected.value) {
    socket.send(JSON.stringify({ event: 'PING', timestamp: Date.now() }));
  }
}

onMounted(() => {
  pingRestServer();
  socket = transportApi.connectWebSocket(
    (msg) => {
      wsMessages.value.unshift(JSON.stringify(msg));
    },
    (connected) => {
      isWsConnected.value = connected;
    }
  );
});
</script>

<template>
  <div class="max-w-md mx-auto my-10 p-6 bg-panelApp border border-borderApp rounded-2xl shadow-glow">
    <h2 class="text-xl font-bold mb-4 text-textBase">BarberOS Connection Monitor</h2>

    <!-- REST Status -->
    <div class="mb-6 p-4 rounded-xl border border-borderApp bg-inputBg">
      <div class="flex items-center justify-between mb-2">
        <span class="text-xs font-mono text-textMuted uppercase">REST Engine Status</span>
        <span
          :class="restStatus ? 'bg-accentEmerald/20 text-accentEmerald border-accentEmerald/30' : 'bg-alertRed/20 text-alertRed border-alertRed/30'"
          class="px-2 py-0.5 text-[10px] font-mono border rounded-full"
        >
          {{ restStatus ? 'ONLINE' : 'OFFLINE' }}
        </span>
      </div>
      <div v-if="restStatus" class="text-xs font-mono text-textBase space-y-1">
        <p>Service: {{ restStatus.service }}</p>
        <p>Version: {{ restStatus.version }}</p>
      </div>
      <p v-else-if="restError" class="text-xs text-alertRed font-mono">{{ restError }}</p>
      <button @click="pingRestServer" class="mt-3 text-xs text-accentEmerald underline">Re-check REST</button>
    </div>

    <!-- WS Status -->
    <div class="p-4 rounded-xl border border-borderApp bg-inputBg">
      <div class="flex items-center justify-between mb-2">
        <span class="text-xs font-mono text-textMuted uppercase">WebSocket Pipeline</span>
        <span
          :class="isWsConnected ? 'bg-accentEmerald/20 text-accentEmerald border-accentEmerald/30' : 'bg-alertRed/20 text-alertRed border-alertRed/30'"
          class="px-2 py-0.5 text-[10px] font-mono border rounded-full"
        >
          {{ isWsConnected ? 'CONNECTED' : 'DISCONNECTED' }}
        </span>
      </div>

      <button
        @click="sendWsPing"
        :disabled="!isWsConnected"
        class="w-full py-2 mb-3 bg-accentEmerald text-[#05080E] font-medium text-xs rounded-lg disabled:opacity-50"
      >
        Send WebSocket Ping
      </button>

      <div class="max-h-32 overflow-y-auto space-y-1 bg-bgApp p-2 rounded border border-borderApp">
        <p class="text-[10px] font-mono text-textMuted">Logs:</p>
        <p v-for="(log, idx) in wsMessages" :key="idx" class="text-[10px] font-mono text-accentEmerald truncate">
          {{ log }}
        </p>
      </div>
    </div>
  </div>
</template>