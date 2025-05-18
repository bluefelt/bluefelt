import { useState } from 'react';
import { useNavigate } from 'react-router-dom';

export default function Register() {
  const [name, setName] = useState('');
  const navigate = useNavigate();

  const submit = async () => {
    const res = await fetch('http://localhost:8000/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: name }),
    });
    const user = await res.json();
    localStorage.setItem('username', user.name);
    navigate('/');
  };

  return (
    <div className="p-4 space-y-4 max-w-md mx-auto">
      <h1 className="text-2xl font-semibold">Register</h1>
      <input
        value={name}
        onChange={(e) => setName(e.target.value)}
        className="border rounded px-2 py-1 w-full"
        placeholder="Your name"
      />
      <button
        disabled={!name}
        onClick={submit}
        className="bg-blue-600 text-white rounded px-3 py-1 disabled:opacity-50"
      >
        Continue
      </button>
    </div>
  );
}
