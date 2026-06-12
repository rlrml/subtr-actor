import { useState } from 'react';
import { Link } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Mail, User, MessageSquare, FileText, Loader2, CheckCircle2, ArrowLeft } from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';
import { AuthCard } from '@/components/ui/GradientCard';
import { useAuth } from '@/hooks/useAuth';
import { API_URL } from '@/api/client';

const MAX_CONTENT_LENGTH = 2000;

const contactFormSchema = z.object({
  name: z.string().min(1, 'Name is required').max(100, 'Name must be 100 characters or less'),
  email: z.string().email('Invalid email address'),
  subject: z.string().min(1, 'Subject is required').max(200, 'Subject must be 200 characters or less'),
  content: z.string().min(1, 'Message is required').max(MAX_CONTENT_LENGTH, `Message must be ${MAX_CONTENT_LENGTH} characters or less`),
  website: z.string().max(0).optional(), // Honeypot
});

type ContactFormData = z.infer<typeof contactFormSchema>;

export default function Contact() {
  const { user } = useAuth();
  const [isLoading, setIsLoading] = useState(false);
  const [isSuccess, setIsSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<ContactFormData>({
    resolver: zodResolver(contactFormSchema),
    defaultValues: {
      name: user?.username || '',
      email: user?.email || '',
      subject: '',
      content: '',
      website: '',
    },
  });

  const contentValue = watch('content') || '';
  const charactersRemaining = MAX_CONTENT_LENGTH - contentValue.length;

  const onSubmit = async (data: ContactFormData) => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await fetch(`${API_URL}/contact`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify(data),
      });

      if (response.status === 429) {
        const result = await response.json();
        setError(`Please wait ${result.retryAfter || 60} seconds before sending another message.`);
        return;
      }

      if (!response.ok) {
        const result = await response.json();
        throw new Error(result.message || 'Failed to send message');
      }

      setIsSuccess(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  if (isSuccess) {
    return (
      <div className="min-h-[80vh] flex items-center justify-center py-12 px-4">
        <SEOHead
          title="Message Sent"
          description="Your message has been sent successfully."
          noIndex
        />
        <div className="w-full max-w-lg">
          <div className="text-center mb-8">
            <div className="inline-flex items-center justify-center w-14 h-14 rounded-full bg-green-500/20 mb-4">
              <CheckCircle2 className="w-7 h-7 text-green-400" />
            </div>
            <h1 className="text-2xl font-bold text-white mb-2">Message Sent!</h1>
            <p className="text-gray-400">
              Thank you for contacting us. We'll get back to you as soon as possible.
            </p>
          </div>

          <AuthCard>
            <p className="text-gray-400 text-sm mb-6 text-center">
              Our team typically responds within 24-48 hours.
            </p>
            <Link to="/">
              <GradientButton className="w-full py-3">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back to Home
              </GradientButton>
            </Link>
          </AuthCard>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-[80vh] flex items-center justify-center py-12 px-4">
      <SEOHead
        title="Contact Us"
        description="Get in touch with the BallCam team. Send us your questions, feedback, or report issues."
      />
      <div className="w-full max-w-lg">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-14 h-14 rounded-full bg-violet-500/20 mb-4">
            <MessageSquare className="w-7 h-7 text-violet-400" />
          </div>
          <h1 className="text-2xl font-bold text-white mb-2">Contact Us</h1>
          <p className="text-gray-400">
            Have a question or feedback? We'd love to hear from you.
          </p>
        </div>

        <AuthCard>
          {error && (
            <div className="mb-6 p-4 rounded-lg bg-red-500/10 border border-red-500/20">
              <p className="text-sm text-red-400">{error}</p>
            </div>
          )}

          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            {/* Name Field */}
            <div>
              <label htmlFor="name" className="block text-sm font-medium text-gray-300 mb-2">
                Your Name
              </label>
              <div className="relative">
                <User className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
                <input
                  {...register('name')}
                  type="text"
                  id="name"
                  placeholder="John Doe"
                  className="w-full pl-10 pr-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
                />
              </div>
              {errors.name && (
                <p className="mt-1 text-sm text-red-400">{errors.name.message}</p>
              )}
            </div>

            {/* Email Field */}
            <div>
              <label htmlFor="email" className="block text-sm font-medium text-gray-300 mb-2">
                Email Address
              </label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
                <input
                  {...register('email')}
                  type="email"
                  id="email"
                  placeholder="you@example.com"
                  className="w-full pl-10 pr-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
                />
              </div>
              {errors.email && (
                <p className="mt-1 text-sm text-red-400">{errors.email.message}</p>
              )}
            </div>

            {/* Subject Field */}
            <div>
              <label htmlFor="subject" className="block text-sm font-medium text-gray-300 mb-2">
                Subject
              </label>
              <div className="relative">
                <FileText className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
                <input
                  {...register('subject')}
                  type="text"
                  id="subject"
                  placeholder="What's this about?"
                  className="w-full pl-10 pr-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
                />
              </div>
              {errors.subject && (
                <p className="mt-1 text-sm text-red-400">{errors.subject.message}</p>
              )}
            </div>

            {/* Message Field */}
            <div>
              <label htmlFor="content" className="block text-sm font-medium text-gray-300 mb-2">
                Message
              </label>
              <textarea
                {...register('content')}
                id="content"
                rows={6}
                placeholder="Tell us what's on your mind..."
                className="w-full px-4 py-3 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all resize-none"
              />
              <div className="flex justify-between mt-1">
                {errors.content ? (
                  <p className="text-sm text-red-400">{errors.content.message}</p>
                ) : (
                  <span />
                )}
                <p className={`text-sm ${charactersRemaining < 100 ? 'text-orange-400' : 'text-gray-500'}`}>
                  {charactersRemaining} characters remaining
                </p>
              </div>
            </div>

            {/* Honeypot field - hidden from users, visible to bots */}
            <input
              {...register('website')}
              type="text"
              name="website"
              autoComplete="off"
              tabIndex={-1}
              aria-hidden="true"
              className="absolute -left-[9999px] top-0 w-0 h-0 opacity-0 pointer-events-none"
            />

            <GradientButton
              type="submit"
              className="w-full py-3"
              disabled={isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-5 h-5 animate-spin mr-2" />
                  Sending...
                </>
              ) : (
                <>
                  <Mail className="w-5 h-5 mr-2" />
                  Send Message
                </>
              )}
            </GradientButton>
          </form>
        </AuthCard>

        <p className="mt-8 text-center text-sm text-gray-500">
          We typically respond within 24-48 hours
        </p>
      </div>
    </div>
  );
}
