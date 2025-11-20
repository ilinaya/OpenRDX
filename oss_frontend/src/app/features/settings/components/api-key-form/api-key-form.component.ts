import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router, ActivatedRoute } from '@angular/router';
import { ApiKeyService } from '../../../../shared/services/api-key.service';
import { CommonModule } from '@angular/common';
import { TranslateModule } from '@ngx-translate/core';

@Component({
  selector: 'app-api-key-form',
  templateUrl: './api-key-form.component.html',
  styleUrls: ['./api-key-form.component.scss'],
  standalone: true,
  imports: [
    CommonModule,
    TranslateModule,
    ReactiveFormsModule,
  ],
})
export class ApiKeyFormComponent implements OnInit {
  apiKeyForm: FormGroup;
  submitting = false;
  error = '';
  success = '';
  generatedKey: string | null = null;
  generatedKeyName: string = '';

  constructor(
    private fb: FormBuilder,
    private apiKeyService: ApiKeyService,
    private router: Router,
    private route: ActivatedRoute
  ) {
    this.apiKeyForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      validity_days: [30, [Validators.required, Validators.min(1), Validators.max(3650)]],
    });
  }

  ngOnInit(): void {
    // API keys cannot be edited, so we always create new ones
  }

  onSubmit(): void {
    if (this.apiKeyForm.invalid) {
      Object.keys(this.apiKeyForm.controls).forEach(key => {
        const control = this.apiKeyForm.get(key);
        control?.markAsTouched();
      });
      return;
    }

    this.submitting = true;
    this.error = '';
    this.success = '';
    this.generatedKey = null;

    const formData = this.apiKeyForm.value;
    this.generatedKeyName = formData.name;

    this.apiKeyService.createApiKey({
      name: formData.name,
      validity_days: formData.validity_days,
    }).subscribe({
      next: (apiKey) => {
        this.submitting = false;
        this.generatedKey = apiKey.key;
        this.success = 'API key generated successfully! Copy it now - you won\'t be able to see it again.';
      },
      error: (err) => {
        this.error = err.error?.error || err.error?.message || 'Failed to generate API key. Please try again later.';
        console.error('Error generating API key:', err);
        this.submitting = false;
      },
    });
  }

  cancel(): void {
    this.router.navigate(['/settings/api-keys']);
  }

  copyToClipboard(text: string): void {
    navigator.clipboard.writeText(text).then(() => {
      // You could show a toast notification here
      alert('API key copied to clipboard!');
    }).catch(err => {
      console.error('Failed to copy to clipboard:', err);
      alert('Failed to copy to clipboard');
    });
  }

  onDone(): void {
    this.router.navigate(['/settings/api-keys']);
  }
}

