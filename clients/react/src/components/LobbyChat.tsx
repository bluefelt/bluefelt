import React, { useState, useRef, useEffect } from 'react';
import type { ChatMessage } from '../types/game-types';

interface LobbyChatProps {
  messages: ChatMessage[];
  currentScope: 'lobby' | 'table';
  currentTableId?: string;
  currentUsername: string;
  onSendMessage: (message: string, scope: 'lobby' | 'table', tableId?: string) => void;
}

export function LobbyChat({
  messages,
  currentScope,
  currentTableId,
  currentUsername,
  onSendMessage,
}: LobbyChatProps) {
  const [inputValue, setInputValue] = useState('');
  const [activeTab, setActiveTab] = useState<'lobby' | 'table'>(currentScope);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  
  // Filter messages by scope
  const filteredMessages = messages.filter(msg => {
    if (activeTab === 'lobby') {
      return msg.scope === 'lobby';
    } else {
      return msg.scope === 'table' && msg.tableId === currentTableId;
    }
  });
  
  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [filteredMessages]);
  
  // Update active tab when scope changes
  useEffect(() => {
    if (currentScope === 'table' && currentTableId) {
      setActiveTab('table');
    }
  }, [currentScope, currentTableId]);
  
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (inputValue.trim()) {
      onSendMessage(
        inputValue.trim(),
        activeTab,
        activeTab === 'table' ? currentTableId : undefined
      );
      setInputValue('');
    }
  };
  
  const formatTimestamp = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };
  
  return (
    <div className="card h-full flex flex-col">
      <div className="mb-4">
        <h3 className="text-lg font-semibold mb-2">Chat</h3>
        
        {/* Tab Selector */}
        <div className="flex gap-2 mb-3">
          <button
            onClick={() => setActiveTab('lobby')}
            className={`px-3 py-1 rounded text-sm ${
              activeTab === 'lobby' 
                ? 'bg-blue-600 text-white' 
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
          >
            Lobby Chat
          </button>
          
          {currentTableId && (
            <button
              onClick={() => setActiveTab('table')}
              className={`px-3 py-1 rounded text-sm ${
                activeTab === 'table' 
                  ? 'bg-blue-600 text-white' 
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              Table Chat
            </button>
          )}
        </div>
      </div>
      
      {/* Messages */}
      <div className="flex-1 overflow-y-auto mb-4 space-y-2 min-h-0">
        {filteredMessages.length === 0 ? (
          <p className="text-gray-400 text-sm text-center py-4">
            No messages yet. Start the conversation!
          </p>
        ) : (
          filteredMessages.map((msg) => (
            <div
              key={msg.id}
              className={`text-sm ${
                msg.sender === currentUsername ? 'text-right' : ''
              }`}
            >
              <div
                className={`inline-block max-w-[80%] rounded-lg px-3 py-2 ${
                  msg.sender === currentUsername
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-700'
                }`}
              >
                {msg.sender !== currentUsername && (
                  <div className="font-semibold text-xs mb-1">{msg.sender}</div>
                )}
                <div className="break-words">{msg.message}</div>
                <div className="text-xs opacity-70 mt-1">
                  {formatTimestamp(msg.timestamp)}
                </div>
              </div>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>
      
      {/* Input */}
      <form onSubmit={handleSubmit} className="flex gap-2">
        <input
          type="text"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          placeholder={`Message ${activeTab} chat...`}
          className="flex-1 bg-gray-700 rounded px-3 py-2 text-sm"
          maxLength={200}
        />
        <button
          type="submit"
          disabled={!inputValue.trim()}
          className="btn btn-sm btn-primary"
        >
          Send
        </button>
      </form>
    </div>
  );
}