import { useState, useEffect, useRef } from 'react'
import Onboarding from './Onboarding'

interface Message {
  id: string
  conversation_id: string
  sender_id: string
  content: string
  message_type: string
  status: string
  created_at: string
  connector_id: string
}

interface Conversation {
  id: string
  title: string
  conversation_type: string
  last_message_at: string | null
}

interface Account {
  connector: string
  display_name: string
  platform_account_id: string
  is_connected: boolean
}

const API = 'http://localhost:3000/api'

function isTelegram(title: string): boolean {
  return title.startsWith('TG:') || title.toLowerCase().includes('telegram')
}

function formatConversationTitle(title: string): string {
  // Transform "TG:8925002788:@username" → "@username"
  if (title.startsWith('TG:')) {
    const parts = title.split(':')
    // parts[0] = "TG", parts[1] = chat_id, parts[2..] = display name (may contain colons)
    if (parts.length >= 3) {
      return parts.slice(2).join(':')
    }
  }
  return title
}

function getConnectorIcon(title: string): string {
  if (isTelegram(title)) return '✈️'
  if (title.toLowerCase().includes('discord')) return '🎮'
  if (title.toLowerCase().includes('whatsapp')) return '📱'
  if (title.toLowerCase().includes('gmail') || title.toLowerCase().includes('email')) return '📧'
  return '💬'
}

function getConnectorColor(title: string): string {
  if (isTelegram(title)) return 'bg-blue-500'
  if (title.toLowerCase().includes('discord')) return 'bg-indigo-500'
  if (title.toLowerCase().includes('whatsapp')) return 'bg-green-500'
  if (title.toLowerCase().includes('gmail') || title.toLowerCase().includes('email')) return 'bg-red-500'
  return 'bg-gray-500'
}

export default function App() {
  const [conversations, setConversations] = useState<Conversation[]>([])
  const [selected, setSelected] = useState<string | null>(null)
  const [messages, setMessages] = useState<Message[]>([])
  const [newMsg, setNewMsg] = useState('')
  const [search, setSearch] = useState('')
  const [showNewChat, setShowNewChat] = useState(false)
  const [newChatTitle, setNewChatTitle] = useState('')
  const [isOnline, setIsOnline] = useState<boolean | null>(null)
  const [_accounts, setAccounts] = useState<Account[]>([])
  const [showOnboarding, setShowOnboarding] = useState(false)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  // Check accounts on mount
  useEffect(() => {
    const checkAccounts = async () => {
      try {
        const r = await fetch(`${API}/accounts`)
        if (r.ok) {
          const d = await r.json()
          const accs = d.data || []
          setAccounts(accs)
          if (accs.length === 0) {
            setShowOnboarding(true)
          }
        }
      } catch {
        // Server not ready yet
      }
    }
    checkAccounts()
  }, [])

  // Poll conversations & health every 2.5 seconds
  useEffect(() => {
    const fetchConversations = async () => {
      try {
        const r = await fetch(`${API}/conversations`)
        if (!r.ok) throw new Error('API non joignable')
        const d = await r.json()
        setConversations(d.data || [])
        setIsOnline(true)
      } catch {
        setIsOnline(false)
      }
    }

    fetchConversations()
    const interval = setInterval(fetchConversations, 2500)
    return () => clearInterval(interval)
  }, [])

  // Poll messages for the active conversation every 1.5 seconds
  useEffect(() => {
    if (!selected) {
      setMessages([])
      return
    }

    const fetchMessages = async () => {
      try {
        const r = await fetch(`${API}/conversations/${selected}/messages`)
        if (r.ok) {
          const d = await r.json()
          setMessages(d.data || [])
        }
      } catch (err) {
        console.error("Erreur lors de la récupération des messages:", err)
      }
    }

    fetchMessages()
    const interval = setInterval(fetchMessages, 1500)
    return () => clearInterval(interval)
  }, [selected])

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const handleSend = async () => {
    if (!newMsg.trim() || !selected) return
    const textToSend = newMsg
    setNewMsg('')

    try {
      await fetch(`${API}/messages`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ conversation_id: selected, content: textToSend, sender_name: 'Vous' }),
      })
      const r = await fetch(`${API}/conversations/${selected}/messages`)
      if (r.ok) {
        const d = await r.json()
        setMessages(d.data || [])
      }
    } catch (err) {
      console.error("Erreur d'envoi:", err)
    }
  }

  const handleCreateChat = async () => {
    if (!newChatTitle.trim()) return
    try {
      const r = await fetch(`${API}/conversations`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title: newChatTitle, conversation_type: 'private' }),
      })
      const d = await r.json()
      if (d.data) {
        setConversations(prev => [d.data, ...prev])
        setSelected(d.data.id)
        setNewChatTitle('')
        setShowNewChat(false)
      }
    } catch (err) {
      console.error("Erreur création conversation:", err)
    }
  }

  const handleOnboardingComplete = () => {
    setShowOnboarding(false)
    // Refresh accounts
    fetch(`${API}/accounts`)
      .then(r => r.json())
      .then(d => setAccounts(d.data || []))
      .catch(() => {})
  }

  // Show onboarding if no accounts
  if (showOnboarding) {
    return <Onboarding onComplete={handleOnboardingComplete} />
  }

  const filtered = conversations.filter(c =>
    c.title.toLowerCase().includes(search.toLowerCase())
  )

  const selectedConv = conversations.find(c => c.id === selected)

  return (
    <div className="flex h-screen bg-gray-950 text-white font-sans">
      {/* Sidebar */}
      <div className="w-80 bg-gray-900 border-r border-gray-800 flex flex-col">
        {/* Header */}
        <div className="p-4 border-b border-gray-800">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <h1 className="text-lg font-bold tracking-tight">Socials</h1>
              <span
                className={`inline-block w-2.5 h-2.5 rounded-full ${
                  isOnline === true ? 'bg-emerald-500' : isOnline === false ? 'bg-rose-500' : 'bg-amber-500 animate-pulse'
                }`}
                title={isOnline === true ? 'Connecté au serveur' : isOnline === false ? 'Serveur déconnecté (lancez cargo run -p socials-server)' : 'Connexion...'}
              />
            </div>
            <button
              onClick={() => setShowNewChat(true)}
              className="w-8 h-8 bg-blue-600 hover:bg-blue-500 rounded-lg flex items-center justify-center text-sm font-bold transition-colors"
            >
              +
            </button>
          </div>
          <div className="relative">
            <input
              type="text"
              placeholder="Rechercher..."
              value={search}
              onChange={e => setSearch(e.target.value)}
              className="w-full bg-gray-800 rounded-lg px-3 py-2 text-sm placeholder-gray-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
          </div>
        </div>

        {/* New chat modal */}
        {showNewChat && (
          <div className="border-b border-gray-800 bg-gray-900">
            <div className="p-3 flex items-center justify-between">
              <span className="text-sm font-medium text-gray-300">Nouvelle conversation</span>
              <button
                onClick={() => { setShowNewChat(false); setNewChatTitle('') }}
                className="text-gray-500 hover:text-gray-300 transition-colors text-lg leading-none"
              >
                ×
              </button>
            </div>
            <div className="px-3 pb-2 text-xs text-gray-500 bg-blue-950/30 mx-3 mb-3 rounded-lg p-2 border border-blue-900/40">
              <div className="flex items-start gap-1.5">
                <span className="text-blue-400 mt-0.5">✈️</span>
                <div>
                  <span className="text-blue-300 font-medium">Telegram :</span> pour qu'un contact apparaisse ici, il doit d'abord envoyer un message à votre bot. Une fois qu'il l'a fait, la conversation apparaît automatiquement.
                </div>
              </div>
            </div>
            <div className="px-3 pb-3 flex gap-2">
              <input
                type="text"
                placeholder="Nom de la conversation..."
                value={newChatTitle}
                onChange={e => setNewChatTitle(e.target.value)}
                onKeyDown={e => e.key === 'Enter' && handleCreateChat()}
                className="flex-1 bg-gray-800 rounded-lg px-3 py-2 text-sm placeholder-gray-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                autoFocus
              />
              <button
                onClick={handleCreateChat}
                className="bg-blue-600 hover:bg-blue-500 px-3 py-2 rounded-lg text-sm font-medium transition-colors"
              >
                Créer
              </button>
            </div>
          </div>
        )}

        {/* Conversations list */}
        <div className="flex-1 overflow-y-auto">
          {filtered.length === 0 ? (
            <div className="p-6 text-gray-500 text-center text-sm">
              Aucune conversation
            </div>
          ) : (
            filtered.map(conv => {
              const icon = getConnectorIcon(conv.title)
              const color = getConnectorColor(conv.title)
              return (
                <div
                  key={conv.id}
                  onClick={() => setSelected(conv.id)}
                  className={`flex items-center gap-3 px-4 py-3 cursor-pointer transition-colors border-l-2 ${
                    selected === conv.id
                      ? 'bg-gray-800 border-blue-500'
                      : 'border-transparent hover:bg-gray-800/50'
                  }`}
                >
                  <div className={`w-10 h-10 rounded-full ${color} flex items-center justify-center text-lg flex-shrink-0`}>
                    {icon}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="font-medium text-sm truncate">{formatConversationTitle(conv.title)}</div>
                    {conv.last_message_at && (
                      <div className="text-xs text-gray-500 mt-0.5">
                        {new Date(conv.last_message_at).toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' })}
                      </div>
                    )}
                  </div>
                </div>
              )
            })
          )}
        </div>
      </div>

      {/* Main chat area */}
      <div className="flex-1 flex flex-col">
        {selected && selectedConv ? (
          <>
            {/* Chat header */}
            <div className="h-14 px-4 flex items-center gap-3 border-b border-gray-800 bg-gray-900/50">
              <div className={`w-8 h-8 rounded-full ${getConnectorColor(selectedConv.title)} flex items-center justify-center text-sm`}>
                {getConnectorIcon(selectedConv.title)}
              </div>
              <div>
                <div className="font-semibold text-sm">{selectedConv.title}</div>
                <div className="text-xs text-gray-500">En ligne</div>
              </div>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-4 space-y-1">
              {messages.map((msg, i) => {
                const isMe = msg.connector_id === 'Vous'
                const showSender = i === 0 || messages[i - 1]?.connector_id !== msg.connector_id
                return (
                  <div key={msg.id} className={`flex flex-col ${isMe ? 'items-end' : 'items-start'} ${showSender ? 'mt-3' : ''}`}>
                    {showSender && (
                      <div className={`text-xs text-gray-500 mb-1 px-1 ${isMe ? 'text-right w-full' : ''}`}>
                        {msg.connector_id}
                      </div>
                    )}
                    <div
                      className={`max-w-md px-4 py-2.5 rounded-2xl text-sm leading-relaxed ${
                        isMe
                          ? 'bg-blue-600 text-white rounded-br-md'
                          : 'bg-gray-800 text-gray-100 rounded-bl-md'
                      }`}
                    >
                      {msg.content}
                      <div className={`text-[10px] mt-1 ${isMe ? 'text-blue-200' : 'text-gray-500'}`}>
                        {new Date(msg.created_at).toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' })}
                      </div>
                    </div>
                  </div>
                )
              })}
              <div ref={messagesEndRef} />
            </div>

            {/* Input */}
            <div className="p-4 border-t border-gray-800 bg-gray-900/30">
              <div className="flex gap-2">
                <input
                  type="text"
                  value={newMsg}
                  onChange={e => setNewMsg(e.target.value)}
                  onKeyDown={e => e.key === 'Enter' && handleSend()}
                  placeholder="Ecrire un message..."
                  className="flex-1 bg-gray-800 rounded-full px-5 py-3 text-sm placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all"
                />
                <button
                  onClick={handleSend}
                  className="bg-blue-600 hover:bg-blue-500 px-6 py-3 rounded-full text-sm font-medium transition-colors"
                >
                  Envoyer
                </button>
              </div>
            </div>
          </>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center text-gray-600 gap-4">
            <div className="text-6xl">💬</div>
            <div className="text-lg">Selectionnez une conversation</div>
            <div className="text-sm text-gray-700">ou creez-en une nouvelle</div>
          </div>
        )}
      </div>
    </div>
  )
}
