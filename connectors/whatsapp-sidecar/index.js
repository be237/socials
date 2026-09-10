const { default: makeWASocket, useMultiFileAuthState, DisconnectReason, fetchLatestBaileysVersion, makeCacheableSignalKeyStore } = require('@whiskeysockets/baileys')
const pino = require('pino')
const express = require('express')
const qrcode = require('qrcode-terminal')
const path = require('path')
const fs = require('fs')

const PORT = process.env.WA_SIDECAR_PORT || 3001
const DATA_DIR = process.env.WA_SIDECAR_DATA || path.join(process.env.HOME || '.', '.local/share/socials/whatsapp')

const app = express()
app.use(express.json())

let sock = null
let qrCode = null
let pairingCode = null
let connectionStatus = 'disconnected' // disconnected | connecting | qr | paired | connected
let statusMessage = ''

const logger = pino({ level: 'silent' })

async function startSock() {
  if (sock) return

  fs.mkdirSync(path.join(DATA_DIR, 'auth_info'), { recursive: true })

  const { state, saveCreds } = await useMultiFileAuthState(path.join(DATA_DIR, 'auth_info'))
  const { version } = await fetchLatestBaileysVersion()

  sock = makeWASocket({
    version,
    auth: {
      creds: state.creds,
      keys: makeCacheableSignalKeyStore(state.keys, logger),
    },
    printQRInTerminal: false,
    logger,
    browser: ['Socials', 'Safari', '1.0'],
  })

  sock.ev.on('creds.update', saveCreds)

  sock.ev.on('connection.update', (update) => {
    const { connection, lastDisconnect, qr } = update

    if (qr) {
      qrCode = qr
      pairingCode = null
      connectionStatus = 'qr'
      statusMessage = 'Scan QR code with your phone'
      qrcode.generate(qr, { small: true })
    }

    if (connection === 'close') {
      const statusCode = lastDisconnect?.error?.output?.statusCode
      const shouldReconnect = statusCode !== DisconnectReason.loggedOut
      console.log(`Connection closed. Status: ${statusCode}. Reconnect: ${shouldReconnect}`)
      sock = null
      qrCode = null
      pairingCode = null
      connectionStatus = 'disconnected'
      statusMessage = lastDisconnect?.error?.message || 'Disconnected'
      if (shouldReconnect) {
        setTimeout(startSock, 3000)
      }
    }

    if (connection === 'open') {
      qrCode = null
      pairingCode = null
      connectionStatus = 'connected'
      statusMessage = 'Connected'
      console.log('WhatsApp connected!')
    }
  })
}

// --- Endpoints ---

app.post('/start', async (req, res) => {
  try {
    if (connectionStatus === 'connected' || connectionStatus === 'qr') {
      return res.json({ ok: true, status: connectionStatus })
    }
    connectionStatus = 'connecting'
    statusMessage = 'Starting connection...'
    startSock()
    // Wait briefly for QR to appear
    await new Promise(r => setTimeout(r, 2000))
    res.json({ ok: true, status: connectionStatus })
  } catch (err) {
    res.status(500).json({ ok: false, error: err.message })
  }
})

app.get('/qr', (req, res) => {
  if (qrCode) {
    res.json({ ok: true, qr: qrCode })
  } else {
    res.json({ ok: false, qr: null, status: connectionStatus })
  }
})

app.post('/pair-code', async (req, res) => {
  const { phoneNumber } = req.body
  if (!phoneNumber) {
    return res.status(400).json({ ok: false, error: 'phoneNumber required' })
  }

  if (!sock) {
    return res.status(400).json({ ok: false, error: 'Not connected. Call /start first.' })
  }

  try {
    // Request pairing code for phone number (8-digit number without +)
    const code = await sock.requestPairingCode(phoneNumber.replace(/\D/g, ''))
    pairingCode = code
    connectionStatus = 'pairing'
    statusMessage = `Pairing code: ${code}`
    res.json({ ok: true, code })
  } catch (err) {
    res.status(500).json({ ok: false, error: err.message })
  }
})

app.get('/status', (req, res) => {
  res.json({
    ok: true,
    status: connectionStatus,
    message: statusMessage,
    user: sock?.user || null,
  })
})

app.get('/chats', async (req, res) => {
  if (!sock || connectionStatus !== 'connected') {
    return res.json({ ok: true, chats: [] })
  }
  try {
    const chats = await sock.store?.chats?.all() || []
    const result = chats.map(c => ({
      id: c.id,
      name: c.name || c.id,
      unreadCount: c.unreadCount || 0,
      lastMessage: c.lastMessage?.message || null,
      lastMessageTime: c.lastMessage?.messageTimestamp || null,
    }))
    res.json({ ok: true, chats: result })
  } catch (err) {
    res.json({ ok: true, chats: [] })
  }
})

app.get('/chats/:id/messages', async (req, res) => {
  if (!sock || connectionStatus !== 'connected') {
    return res.json({ ok: true, messages: [] })
  }
  const chatId = req.params.id
  try {
    const msgs = await sock.loadMessages(chatId, 50)
    const result = (msgs || []).map(m => ({
      id: m.key.id,
      from: m.key.remoteJid,
      sender: m.key.participant || m.key.remoteJid,
      content: m.message?.conversation || m.message?.extendedTextMessage?.text || '',
      timestamp: m.messageTimestamp,
      fromMe: m.key.fromMe,
    })).reverse()
    res.json({ ok: true, messages: result })
  } catch (err) {
    res.json({ ok: true, messages: [] })
  }
})

app.post('/send', async (req, res) => {
  const { to, text } = req.body
  if (!sock || connectionStatus !== 'connected') {
    return res.status(400).json({ ok: false, error: 'Not connected' })
  }
  if (!to || !text) {
    return res.status(400).json({ ok: false, error: 'to and text required' })
  }
  try {
    const result = await sock.sendMessage(to, { text })
    res.json({ ok: true, id: result.key.id })
  } catch (err) {
    res.status(500).json({ ok: false, error: err.message })
  }
})

app.post('/disconnect', async (req, res) => {
  try {
    if (sock) {
      sock.end()
      sock = null
    }
    qrCode = null
    pairingCode = null
    connectionStatus = 'disconnected'
    statusMessage = 'Disconnected'
    res.json({ ok: true })
  } catch (err) {
    res.status(500).json({ ok: false, error: err.message })
  }
})

app.listen(PORT, '127.0.0.1', () => {
  console.log(`WhatsApp sidecar running on http://127.0.0.1:${PORT}`)
})
