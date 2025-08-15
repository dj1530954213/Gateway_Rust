import { ref, readonly, onMounted, onUnmounted, nextTick } from 'vue'

interface UseWebSocketOptions {
  autoConnect?: boolean
  reconnectInterval?: number
  maxReconnectAttempts?: number
  heartbeatInterval?: number
  onOpen?: () => void
  onClose?: () => void
  onError?: (error: Event) => void
  onMessage?: (message: string) => void
}

export function useWebSocket(url: string, options: UseWebSocketOptions = {}) {
  const {
    autoConnect = true,
    reconnectInterval = 5000,
    maxReconnectAttempts = 10,
    heartbeatInterval = 30000,
    onOpen,
    onClose,
    onError,
    onMessage,
  } = options

  const socket = ref<WebSocket | null>(null)
  const isConnected = ref(false)
  const isConnecting = ref(false)
  const lastMessage = ref<string>('')
  const reconnectAttempts = ref(0)

  let reconnectTimer: NodeJS.Timeout | null = null
  let heartbeatTimer: NodeJS.Timeout | null = null

  const connect = async () => {
    if (isConnecting.value || isConnected.value) {
      return
    }

    isConnecting.value = true

    try {
      // Determine WebSocket URL
      const wsUrl = new URL(url, window.location.origin)
      wsUrl.protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'

      socket.value = new WebSocket(wsUrl.toString())

      socket.value.onopen = () => {
        isConnected.value = true
        isConnecting.value = false
        reconnectAttempts.value = 0

        console.log('WebSocket connected:', url)

        // Start heartbeat
        startHeartbeat()

        onOpen?.()
      }

      socket.value.onclose = () => {
        isConnected.value = false
        isConnecting.value = false

        console.log('WebSocket disconnected:', url)

        // Stop heartbeat
        stopHeartbeat()

        onClose?.()

        // Auto reconnect
        if (reconnectAttempts.value < maxReconnectAttempts) {
          scheduleReconnect()
        }
      }

      socket.value.onerror = error => {
        console.error('WebSocket error:', error)
        isConnecting.value = false

        onError?.(error)
      }

      socket.value.onmessage = event => {
        lastMessage.value = event.data
        onMessage?.(event.data)
      }
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error)
      isConnecting.value = false
    }
  }

  const disconnect = () => {
    if (socket.value) {
      socket.value.close()
      socket.value = null
    }

    stopHeartbeat()
    stopReconnect()

    isConnected.value = false
    isConnecting.value = false
  }

  const send = (data: string | object) => {
    if (!isConnected.value) {
      console.warn('WebSocket is not connected')
      return false
    }

    if (!socket.value) {
      console.warn('WebSocket socket is not available')
      return false
    }

    try {
      const message = typeof data === 'string' ? data : JSON.stringify(data)
      socket.value.send(message)
      return true
    } catch (error) {
      console.error('Failed to send WebSocket message:', error)
      return false
    }
  }

  const scheduleReconnect = () => {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
    }

    reconnectAttempts.value++

    console.log(
      `Scheduling WebSocket reconnect attempt ${reconnectAttempts.value}/${maxReconnectAttempts}`
    )

    reconnectTimer = setTimeout(() => {
      connect()
    }, reconnectInterval)
  }

  const stopReconnect = () => {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }
  }

  const startHeartbeat = () => {
    if (heartbeatTimer) {
      clearInterval(heartbeatTimer)
    }

    heartbeatTimer = setInterval(() => {
      if (isConnected.value) {
        send({ type: 'ping', timestamp: Date.now() })
      }
    }, heartbeatInterval)
  }

  const stopHeartbeat = () => {
    if (heartbeatTimer) {
      clearInterval(heartbeatTimer)
      heartbeatTimer = null
    }
  }

  // Subscribe to specific message types
  const subscribe = (messageType: string, callback: (data: any) => void) => {
    const unsubscribe = () => {
      // Implementation would depend on message format
    }

    // Listen for messages of specific type
    const messageHandler = (message: string) => {
      try {
        const data = JSON.parse(message)
        if (data.type === messageType) {
          callback(data.data || data)
        }
      } catch (error) {
        // Ignore parsing errors for non-JSON messages
      }
    }

    // Add to message listeners (simplified implementation)
    onMessage?.(messageHandler)

    return unsubscribe
  }

  // Auto-connect on mount (with Mock mode support)
  onMounted(() => {
    if (autoConnect) {
      nextTick(() => {
        connect()
      })
    }
  })

  // Clean up on unmount
  onUnmounted(() => {
    disconnect()
  })

  return {
    socket: readonly(socket),
    isConnected: readonly(isConnected),
    isConnecting: readonly(isConnecting),
    lastMessage: readonly(lastMessage),
    reconnectAttempts: readonly(reconnectAttempts),
    connect,
    disconnect,
    send,
    subscribe,
  }
}
