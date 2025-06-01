import { Component, OnInit } from '@angular/core';
import {FormBuilder, FormGroup, ReactiveFormsModule, Validators} from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import {RadsecService} from "../../../../../shared/services/radsec.service";

@Component({
  selector: 'app-radsec-source-form',
  templateUrl: './radsec-source-form.component.html',
  imports: [
    ReactiveFormsModule,
  ],
  styleUrls: ['./radsec-source-form.component.scss'],
})
export class RadsecSourceFormComponent implements OnInit {
  sourceForm: FormGroup;
  isEditMode = false;
  loading = false;
  submitting = false;
  error = '';
  sourceId?: number;

  constructor(
    private fb: FormBuilder,
    private radsecService: RadsecService,
    private router: Router,
    private route: ActivatedRoute
  ) {
    this.sourceForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      description: [''],
      source_subnets: [[]],
      tls_key: ['', [Validators.required]],
      tls_cert: ['', [Validators.required]],
    });
  }

  ngOnInit(): void {
    this.sourceId = this.route.snapshot.params['id'];
    if (this.sourceId) {
      this.isEditMode = true;
      this.loadSource();
    }
  }

  loadSource(): void {
    if (!this.sourceId) return;

    this.loading = true;
    this.radsecService.getSource(this.sourceId)
      .subscribe({
        next: (source) => {
          this.sourceForm.patchValue(source);
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load source. Please try again later.';
          console.error('Error loading source:', err);
          this.loading = false;
        }
      });
  }

  onSubmit(): void {
    if (this.sourceForm.invalid) {
      return;
    }

    this.submitting = true;
    const sourceData = this.sourceForm.value;

    const request = this.isEditMode
      ? this.radsecService.updateSource(this.sourceId!, sourceData)
      : this.radsecService.createSource(sourceData);

    request.subscribe({
      next: () => {
        this.router.navigate(['../'], { relativeTo: this.route });
      },
      error: (err) => {
        this.error = `Failed to ${this.isEditMode ? 'update' : 'create'} source. Please try again later.`;
        console.error(`Error ${this.isEditMode ? 'updating' : 'creating'} source:`, err);
        this.submitting = false;
      }
    });
  }

  cancel(): void {
    this.router.navigate(['../'], { relativeTo: this.route });
  }
}
