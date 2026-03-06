import { NextRequest, NextResponse } from 'next/server'
import { createClient } from '@/lib/supabase-server'
import { fetchQuoteData } from '@/lib/stocks'
import { analyzeStock } from '@/lib/openrouter'

function extractTickerFromPrompt(prompt: string): string {
  const parentheticalMatch = prompt.match(/\(([A-Z0-9.\-]+)\)/i);
  if (parentheticalMatch) {
    return parentheticalMatch[1].toUpperCase();
  }
  const words = prompt.split(/[\s,!?]+/); 
  const uppercaseWord = words.find(w => /^[A-Z0-9.\-]{2,8}$/.test(w));
  if (uppercaseWord) {
    return uppercaseWord.toUpperCase();
  }
  return prompt.trim().toUpperCase();
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    
    // TAMBAHAN: Tangkap opsi skipAI
    const { ticker = '', customPrompt = '', agentType = 'fundamental', model: bodyModel = '', skipAI = false } = body

    const model = bodyModel || 'google/gemini-2.5-flash-lite'
    const promptText = (customPrompt || ticker).trim()

    if (!promptText) {
      return NextResponse.json({ success: false, error: 'Prompt or ticker is required' }, { status: 400 })
    }

    const tickerSymbol = extractTickerFromPrompt(promptText);
    
    // 1. Ambil data mentah + kalkulasi kuantitatif menggunakan ticker hasil ekstrak
    const stockData = await fetchQuoteData(tickerSymbol);

    if (!stockData) {
      return NextResponse.json({ 
         success: false, 
         error: `Failed to fetch stock data for ${tickerSymbol}.` 
      }, { status: 404 })
    }

    // TAMBAHAN: Jika hanya butuh data grafik (skipAI = true), langsung kembalikan data tanpa panggil AI
    if (skipAI) {
      return NextResponse.json({ success: true, analysis: '', data: stockData })
    }

    // 2. Lempar ke Agen AI
    const openRouterKey = process.env.OPENROUTER_API_KEY
    let aiAnalysis = ''

    if (openRouterKey) {
      try {
        aiAnalysis = await analyzeStock(tickerSymbol, stockData, openRouterKey, agentType, model)
      } catch (aiError: any) {
        aiAnalysis = 'AI analysis is currently unavailable due to a connection issue.';
      }
    } else {
      aiAnalysis = 'OpenRouter API Key is missing. AI analysis is not available.';
    }

    // 3. Simpan riwayat ke Supabase secara diam-diam
    try {
      const supabase = await createClient()
      const { data: { user } } = await supabase.auth.getUser()
      
      if (user) {
        const { data: chatData, error: chatError } = await supabase
          .from('chats')
          .insert({
            user_id: user.id,
            title: `Analysis: ${tickerSymbol}`
          })
          .select()
          .single()

        if (!chatError && chatData) {
          await supabase.from('messages').insert([
            { chat_id: chatData.id, role: 'user', content: promptText },
            { chat_id: chatData.id, role: 'assistant', content: aiAnalysis }
          ])
        }
      }
    } catch (dbError) {
      console.error("[API] Failed to save history to Supabase:", dbError)
    }

    return NextResponse.json({ success: true, analysis: aiAnalysis, data: stockData })

  } catch (error: any) {
    return NextResponse.json({ success: false, error: error.message || 'Internal Server Error' }, { status: 500 })
  }
}