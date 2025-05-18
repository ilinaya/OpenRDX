import { Component, OnInit } from '@angular/core';
import {FormBuilder, FormGroup, ReactiveFormsModule, Validators} from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { AdminService } from '../../../../shared/services/admin.service';
import { AdminGroup } from '../../../../shared/models/admin.model';

@Component({
  selector: 'app-admin-group-form',
  templateUrl: './admin-group-form.component.html',
  imports: [
    ReactiveFormsModule,
  ],
  styleUrls: ['./admin-group-form.component.scss'],
})
export class AdminGroupFormComponent implements OnInit {
  groupForm: FormGroup;
  isEditMode = false;
  loading = false;
  submitting = false;
  error = '';
  groupId?: number;

  constructor(
    private fb: FormBuilder,
    private adminService: AdminService,
    private router: Router,
    private route: ActivatedRoute
  ) {
    this.groupForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(150)]],
      description: ['', Validators.maxLength(200)]
    });
  }

  ngOnInit(): void {
    this.groupId = this.route.snapshot.params['id'];
    if (this.groupId) {
      this.isEditMode = true;
      this.loadGroup();
    }
  }

  loadGroup(): void {
    if (!this.groupId) return;

    this.loading = true;
    this.adminService.getAdminGroup(this.groupId)
      .subscribe({
        next: (group) => {
          this.groupForm.patchValue(group);
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load admin group. Please try again later.';
          console.error('Error loading admin group:', err);
          this.loading = false;
        }
      });
  }

  onSubmit(): void {
    if (this.groupForm.invalid) {
      return;
    }

    this.submitting = true;
    const groupData = this.groupForm.value;

    const request = this.isEditMode
      ? this.adminService.updateAdminGroup(this.groupId!, groupData)
      : this.adminService.createAdminGroup(groupData);

    request.subscribe({
      next: () => {
        this.router.navigate(['../'], { relativeTo: this.route });
      },
      error: (err) => {
        this.error = `Failed to ${this.isEditMode ? 'update' : 'create'} admin group. Please try again later.`;
        console.error(`Error ${this.isEditMode ? 'updating' : 'creating'} admin group:`, err);
        this.submitting = false;
      }
    });
  }

  cancel(): void {
    this.router.navigate(['../'], { relativeTo: this.route });
  }
}
